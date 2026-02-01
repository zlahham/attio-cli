use crate::cache;
use crate::client::AttioClient;
use crate::models::Note;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::panic;

#[derive(PartialEq)]
enum InputMode {
    Normal,
    Search,
}

fn log_debug(msg: &str) {
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/attio-cli.log")
    {
        let _ = writeln!(file, "{}", msg);
        let _ = file.flush();
    }
}

pub async fn run_list_tui(client: AttioClient, cache_limit_mb: u64) -> Result<(), Box<dyn Error>> {
    log_debug("--- SESSION START ---");

    panic::set_hook(Box::new(|info| {
        let msg = format!("CRITICAL PANIC: {}", info);
        log_debug(&msg);
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        eprintln!(
            "\r\n[TUI Error] The application crashed. Terminal restored.\r\n{}\r\n",
            msg
        );
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, client, cache_limit_mb).await;

    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    let _ = disable_raw_mode();
    let _ = terminal.show_cursor();

    res
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    client: AttioClient,
    cache_limit_mb: u64,
) -> Result<(), Box<dyn Error>> {
    let mut offset = 0;
    let mut all_notes: Vec<Note> = Vec::new(); // Complete cache
    let mut cache_size_bytes: usize = 0; // Track cache size in bytes
    let cache_limit_bytes = (cache_limit_mb as usize) * 1024 * 1024; // Convert MB to bytes
    let mut error_msg: Option<String> = None;
    let mut total_fetched = 0;
    let mut input_mode = InputMode::Normal;
    let mut search_query = String::new();
    let mut search_offset = 0; // Separate offset for search results pagination
    let mut is_fetching_all = false;

    // Calculate initial limit based on terminal size
    // Overhead: 3 (help block) + 2 (table borders) + 1 (table header) = 6 lines
    let calculate_limit = |terminal: &mut Terminal<CrosstermBackend<io::Stdout>>| -> u32 {
        let size = terminal.size().unwrap_or_default();
        let height = size.height.saturating_sub(7) as u32;
        // Cap limit at 50. Attio's notes endpoint seems to have a lower limit than 100.
        let val = height.clamp(1, 50);
        log_debug(&format!(
            "Calculated limit: {} (Terminal height: {})",
            val, size.height
        ));
        val
    };

    let mut limit = calculate_limit(terminal);

    // Helper to add notes to cache with deduplication and size limit
    // Returns (added_count, limit_reached)
    let add_to_cache = |cache: &mut Vec<Note>,
                        cache_size: &mut usize,
                        new_notes: Vec<Note>,
                        limit: usize|
     -> (usize, bool) {
        let mut added = 0;
        let mut limit_reached = false;
        for note in new_notes {
            // Only add if not already in cache
            if !cache.iter().any(|n| n.id.note_id == note.id.note_id) {
                let note_size = cache::estimate_note_size(&note);
                // Check if adding this note would exceed the limit
                if *cache_size + note_size <= limit {
                    *cache_size += note_size;
                    cache.push(note);
                    added += 1;
                } else {
                    // Cache limit reached, stop adding
                    log_debug(&format!(
                        "Cache limit reached: {} bytes / {} bytes",
                        *cache_size, limit
                    ));
                    limit_reached = true;
                    break;
                }
            }
        }
        (added, limit_reached)
    };

    // Helper for rendering
    let draw_screen = |terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
                       all_notes: &[Note],
                       error_msg: &Option<String>,
                       offset: u32,
                       search_offset: u32,
                       limit: u32,
                       _total_fetched: usize,
                       loading: bool,
                       search_query: &str,
                       input_mode: &InputMode,
                       is_fetching_all: bool,
                       cache_size_bytes: usize,
                       cache_limit_bytes: usize|
     -> Result<(), io::Error> {
        // Calculate cache usage
        let cache_mb = cache_size_bytes as f64 / (1024.0 * 1024.0);
        let limit_mb = cache_limit_bytes as f64 / (1024.0 * 1024.0);
        let usage_percent = (cache_size_bytes as f64 / cache_limit_bytes as f64) * 100.0;

        // Color code based on usage
        let cache_color = if usage_percent < 70.0 {
            Color::Green
        } else if usage_percent < 90.0 {
            Color::Yellow
        } else {
            Color::Red
        };
        // In search mode, filter all cached notes and paginate through results
        // In normal mode, show a slice of cached notes based on offset
        let (display_notes, current_page, total_matches): (Vec<&Note>, u32, Option<usize>) =
            if !search_query.is_empty() {
                // Search mode: filter all notes and paginate through filtered results
                let query_lower = search_query.to_lowercase();
                let mut filtered: Vec<&Note> = all_notes
                    .iter()
                    .filter(|note| {
                        note.title.to_lowercase().contains(&query_lower)
                            || note.content_plaintext.to_lowercase().contains(&query_lower)
                    })
                    .collect();

                let total = filtered.len();
                let page = (search_offset / limit.max(1)) + 1;

                // Paginate filtered results
                let start = search_offset as usize;
                let end = (start + limit as usize).min(filtered.len());
                filtered = filtered[start..end].to_vec();

                (filtered, page, Some(total))
            } else {
                // Normal mode: show slice of cached notes
                let start = offset as usize;
                let end = (start + limit as usize).min(all_notes.len());
                let slice: Vec<&Note> = all_notes[start..end].iter().collect();
                let page = (offset / limit.max(1)) + 1;

                (slice, page, None)
            };

        terminal.draw(|f| {
            // Dynamic layout based on search mode
            let chunks = if input_mode == &InputMode::Search || !search_query.is_empty() {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3), // Search box
                        Constraint::Min(0),    // Notes table
                        Constraint::Length(3), // Help footer
                    ])
                    .split(f.area())
            } else {
                Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Min(0),    // Notes table
                        Constraint::Length(3), // Help footer
                    ])
                    .split(f.area())
            };

            let (table_chunk, help_chunk) =
                if input_mode == &InputMode::Search || !search_query.is_empty() {
                    // Render search box
                    let search_text = if input_mode == &InputMode::Search {
                        format!("🔍 {}_", search_query) // Show cursor
                    } else {
                        format!("🔍 {} (Press / to search again)", search_query)
                    };

                    let search_style = if input_mode == &InputMode::Search {
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Yellow)
                    };

                    let search_widget = Paragraph::new(search_text).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Search ")
                            .style(search_style),
                    );
                    f.render_widget(search_widget, chunks[0]);
                    (chunks[1], chunks[2])
                } else {
                    (chunks[0], chunks[1])
                };

            let cache_info = format!("{:.1}MB / {:.0}MB", cache_mb, limit_mb);

            let title_text = if let Some(total) = total_matches {
                format!(
                    " Notes - {} matches from {} cached | Cache: {} (Page {}) ",
                    total,
                    all_notes.len(),
                    cache_info,
                    current_page
                )
            } else if is_fetching_all {
                format!(
                    " Notes - Fetching all... ({} cached) | Cache: {} ",
                    all_notes.len(),
                    cache_info
                )
            } else {
                format!(
                    " Notes - {} cached | Cache: {} (Page {}) ",
                    all_notes.len(),
                    cache_info,
                    current_page
                )
            };

            if loading {
                f.render_widget(
                    Paragraph::new("Loading notes...")
                        .block(Block::default().borders(Borders::ALL).title(" Status ")),
                    table_chunk,
                );
            } else if let Some(msg) = error_msg {
                f.render_widget(
                    Paragraph::new(msg.as_str()).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Error ")
                            .style(Style::default().fg(Color::Red)),
                    ),
                    table_chunk,
                );
            } else {
                let rows = display_notes.iter().map(|n| {
                    let mut content = n.content_plaintext.replace('\n', " ");
                    // Increased truncation limit significantly to utilize width
                    if content.chars().count() > 500 {
                        content = content.chars().take(497).collect::<String>() + "...";
                    }
                    Row::new(vec![
                        Cell::from(
                            n.id.note_id.clone().chars().take(8).collect::<String>() + "...",
                        ),
                        Cell::from(n.title.clone()),
                        Cell::from(content),
                    ])
                });

                let table = Table::new(
                    rows,
                    [
                        Constraint::Length(12),
                        Constraint::Percentage(25),
                        Constraint::Fill(1), // Use remaining space
                    ],
                )
                .header(
                    Row::new(vec!["ID", "Title", "Content"]).style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                )
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(title_text)
                        .title_style(Style::default().add_modifier(Modifier::BOLD))
                        .border_style(Style::default().fg(cache_color)),
                );

                f.render_widget(table, table_chunk);
            }

            // Footer with arrows and page info
            let footer_content = if input_mode == &InputMode::Search {
                Line::from(vec![
                    Span::styled(
                        " Type ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("to search  "),
                    Span::styled(
                        " Backspace ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("to delete  "),
                    Span::styled(
                        " [Esc] ",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("Exit search"),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        " ← ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("Prev  "),
                    Span::styled(
                        " → ",
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("Next  "),
                    Span::styled(
                        " [/] ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("Search  "),
                    Span::styled(
                        " [Ctrl+A] ",
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("Fetch All  "),
                    Span::styled(
                        " [Q] ",
                        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("Quit"),
                ])
            };

            let help = Paragraph::new(footer_content)
                .block(Block::default().borders(Borders::ALL).title(" Controls "));
            f.render_widget(help, help_chunk);
        })?;
        Ok(())
    };

    // Initial fetch
    draw_screen(
        terminal,
        &all_notes,
        &error_msg,
        offset,
        search_offset,
        limit,
        total_fetched,
        true,
        &search_query,
        &input_mode,
        is_fetching_all,
        cache_size_bytes,
        cache_limit_bytes,
    )?;
    match client.list_notes(Some(limit), Some(offset)).await {
        Ok(resp) => {
            total_fetched = resp.data.len();
            let _ = add_to_cache(
                &mut all_notes,
                &mut cache_size_bytes,
                resp.data,
                cache_limit_bytes,
            );
        }
        Err(e) => error_msg = Some(e.to_string()),
    }

    loop {
        draw_screen(
            terminal,
            &all_notes,
            &error_msg,
            offset,
            search_offset,
            limit,
            total_fetched,
            false,
            &search_query,
            &input_mode,
            is_fetching_all,
            cache_size_bytes,
            cache_limit_bytes,
        )?;

        if event::poll(std::time::Duration::from_millis(200))? {
            match event::read()? {
                Event::Resize(_, _) => {
                    limit = calculate_limit(terminal);
                    // No need to re-fetch, just re-render with new limit
                }
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') if input_mode == InputMode::Normal => return Ok(()),
                    KeyCode::Esc => {
                        if input_mode == InputMode::Search {
                            input_mode = InputMode::Normal;
                            search_query.clear();
                            search_offset = 0;
                        } else {
                            return Ok(());
                        }
                    }
                    KeyCode::Char('/') if input_mode == InputMode::Normal => {
                        input_mode = InputMode::Search;
                        search_offset = 0;
                    }
                    KeyCode::Char('a')
                        if input_mode == InputMode::Normal
                            && key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                    {
                        // Fetch all notes
                        is_fetching_all = true;
                        let mut fetch_offset = 0u32;
                        let fetch_limit = 50u32; // Attio's API has a max limit around 50

                        loop {
                            draw_screen(
                                terminal,
                                &all_notes,
                                &error_msg,
                                offset,
                                search_offset,
                                limit,
                                total_fetched,
                                false,
                                &search_query,
                                &input_mode,
                                is_fetching_all,
                                cache_size_bytes,
                                cache_limit_bytes,
                            )?;

                            match client
                                .list_notes(Some(fetch_limit), Some(fetch_offset))
                                .await
                            {
                                Ok(resp) => {
                                    let fetched = resp.data.len();
                                    let (_added, limit_reached) = add_to_cache(
                                        &mut all_notes,
                                        &mut cache_size_bytes,
                                        resp.data,
                                        cache_limit_bytes,
                                    );

                                    if limit_reached {
                                        // Cache limit reached
                                        error_msg = Some(format!(
                                            "Cache limit reached ({:.1}MB / {:.0}MB). Stopped fetching.",
                                            cache_size_bytes as f64 / (1024.0 * 1024.0),
                                            cache_limit_bytes as f64 / (1024.0 * 1024.0)
                                        ));
                                        break;
                                    }
                                    if fetched < fetch_limit as usize {
                                        // No more notes to fetch
                                        break;
                                    }
                                    // Continue fetching even if added == 0 (all duplicates), as long as we got a full page
                                    fetch_offset += fetch_limit;
                                }
                                Err(e) => {
                                    error_msg = Some(format!("Error fetching all: {}", e));
                                    break;
                                }
                            }
                        }

                        is_fetching_all = false;
                    }
                    KeyCode::Char(c) if input_mode == InputMode::Search => {
                        search_query.push(c);
                        search_offset = 0; // Reset to first page of results
                    }
                    KeyCode::Backspace if input_mode == InputMode::Search => {
                        search_query.pop();
                        search_offset = 0; // Reset to first page of results
                    }
                    KeyCode::Right => {
                        if !search_query.is_empty() {
                            // In search mode: paginate through filtered results
                            let query_lower = search_query.to_lowercase();
                            let filtered_count = all_notes
                                .iter()
                                .filter(|note| {
                                    note.title.to_lowercase().contains(&query_lower)
                                        || note
                                            .content_plaintext
                                            .to_lowercase()
                                            .contains(&query_lower)
                                })
                                .count();

                            if search_offset + limit < filtered_count as u32 {
                                search_offset += limit;
                                terminal.clear()?; // Clear artifacts when changing pages
                            }
                        } else if input_mode == InputMode::Normal {
                            // In normal mode: check if we can go forward
                            let next_offset = offset + limit;

                            if next_offset < all_notes.len() as u32 {
                                // Already have data in cache, safe to move forward
                                offset = next_offset;
                                terminal.clear()?; // Clear artifacts when changing pages
                            } else if total_fetched == limit as usize {
                                // Try to fetch more from API
                                terminal.clear()?;
                                draw_screen(
                                    terminal,
                                    &all_notes,
                                    &error_msg,
                                    offset, // Keep current offset during fetch
                                    search_offset,
                                    limit,
                                    total_fetched,
                                    true,
                                    &search_query,
                                    &input_mode,
                                    is_fetching_all,
                                    cache_size_bytes,
                                    cache_limit_bytes,
                                )?;
                                match client.list_notes(Some(limit), Some(next_offset)).await {
                                    Ok(resp) => {
                                        total_fetched = resp.data.len();
                                        let (_added, limit_reached) = add_to_cache(
                                            &mut all_notes,
                                            &mut cache_size_bytes,
                                            resp.data,
                                            cache_limit_bytes,
                                        );

                                        // Only move forward if we have data at the next offset
                                        if next_offset < all_notes.len() as u32 {
                                            offset = next_offset;
                                            error_msg = None;
                                            terminal.clear()?;
                                        } else if limit_reached {
                                            error_msg = Some(
                                                "Cache limit reached. Not caching new notes."
                                                    .to_string(),
                                            );
                                        }
                                        // If total_fetched == 0, we're at the end, don't move
                                    }
                                    Err(e) => error_msg = Some(e.to_string()),
                                }
                            }
                            // If neither condition is true, we're at the end - don't move
                        }
                    }
                    KeyCode::Left => {
                        if !search_query.is_empty() {
                            // In search mode: paginate through filtered results
                            if search_offset > 0 {
                                search_offset = search_offset.saturating_sub(limit);
                                terminal.clear()?; // Clear artifacts when changing pages
                            }
                        } else if input_mode == InputMode::Normal && offset > 0 {
                            // In normal mode: just move offset (already in cache)
                            offset = offset.saturating_sub(limit);
                            terminal.clear()?; // Clear artifacts when changing pages
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
