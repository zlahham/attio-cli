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

pub async fn run_list_tui(client: AttioClient) -> Result<(), Box<dyn Error>> {
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

    let res = run_app(&mut terminal, client).await;

    let _ = execute!(io::stdout(), LeaveAlternateScreen);
    let _ = disable_raw_mode();
    let _ = terminal.show_cursor();

    res
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    client: AttioClient,
) -> Result<(), Box<dyn Error>> {
    let mut offset = 0;
    let mut notes: Vec<Note> = Vec::new();
    let mut error_msg: Option<String> = None;
    let mut total_fetched = 0;

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

    // Helper for rendering
    let draw_screen = |terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
                       notes: &[Note],
                       error_msg: &Option<String>,
                       offset: u32,
                       limit: u32,
                       _total_fetched: usize,
                       loading: bool|
     -> Result<(), io::Error> {
        let current_page = (offset / limit.max(1)) + 1;

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0), Constraint::Length(3)])
                .split(f.area());

            let title_text = format!(" Notes (Page {}, Offset: {}) ", current_page, offset);

            if loading {
                f.render_widget(
                    Paragraph::new("Loading notes...")
                        .block(Block::default().borders(Borders::ALL).title(" Status ")),
                    chunks[0],
                );
            } else if let Some(msg) = error_msg {
                f.render_widget(
                    Paragraph::new(msg.as_str()).block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title(" Error ")
                            .style(Style::default().fg(Color::Red)),
                    ),
                    chunks[0],
                );
            } else {
                let rows = notes.iter().map(|n| {
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
                        .title_style(
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                );

                f.render_widget(table, chunks[0]);
            }

            // Footer with arrows and page info
            let footer_content = Line::from(vec![
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
                Span::raw(format!("|  Page {}  ", current_page)),
                Span::styled(
                    " [Q] ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw("Quit"),
            ]);

            let help = Paragraph::new(footer_content)
                .block(Block::default().borders(Borders::ALL).title(" Controls "));
            f.render_widget(help, chunks[1]);
        })?;
        Ok(())
    };

    // Initial fetch
    draw_screen(
        terminal,
        &notes,
        &error_msg,
        offset,
        limit,
        total_fetched,
        true,
    )?;
    match client.list_notes(Some(limit), Some(offset)).await {
        Ok(resp) => {
            notes = resp.data;
            total_fetched = notes.len();
        }
        Err(e) => error_msg = Some(e.to_string()),
    }

    loop {
        draw_screen(
            terminal,
            &notes,
            &error_msg,
            offset,
            limit,
            total_fetched,
            false,
        )?;

        if event::poll(std::time::Duration::from_millis(200))? {
            match event::read()? {
                Event::Resize(_, _) => {
                    limit = calculate_limit(terminal);
                    // Re-fetch on resize to fill the new space
                    draw_screen(
                        terminal,
                        &notes,
                        &error_msg,
                        offset,
                        limit,
                        total_fetched,
                        true,
                    )?;
                    if let Ok(resp) = client.list_notes(Some(limit), Some(offset)).await {
                        notes = resp.data;
                        total_fetched = notes.len();
                    }
                }
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Right => {
                        if total_fetched == limit as usize {
                            offset += limit;
                            draw_screen(
                                terminal,
                                &notes,
                                &error_msg,
                                offset,
                                limit,
                                total_fetched,
                                true,
                            )?;
                            match client.list_notes(Some(limit), Some(offset)).await {
                                Ok(resp) => {
                                    notes = resp.data;
                                    total_fetched = notes.len();
                                    error_msg = None;
                                }
                                Err(e) => error_msg = Some(e.to_string()),
                            }
                        }
                    }
                    KeyCode::Left => {
                        if offset > 0 {
                            offset = offset.saturating_sub(limit);
                            draw_screen(
                                terminal,
                                &notes,
                                &error_msg,
                                offset,
                                limit,
                                total_fetched,
                                true,
                            )?;
                            match client.list_notes(Some(limit), Some(offset)).await {
                                Ok(resp) => {
                                    notes = resp.data;
                                    total_fetched = notes.len();
                                    error_msg = None;
                                }
                                Err(e) => error_msg = Some(e.to_string()),
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
