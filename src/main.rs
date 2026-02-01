mod cache;
mod client;
mod models;
mod tui;

use clap::{Parser, Subcommand};
use client::AttioClient;
use dotenvy::dotenv;
use std::env;
use std::error::Error;

use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "attio", author, version, about = "A CLI tool for Attio CRM", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with Attio
    Auth {
        /// Your Attio API Token
        token: String,
    },
    /// Note related actions
    Notes {
        #[command(subcommand)]
        action: NoteCommands,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Set a configuration value
    Set {
        /// Configuration key (e.g., cache-limit-mb)
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get a configuration value
    Get {
        /// Configuration key (e.g., cache-limit-mb)
        key: String,
    },
    /// List all configuration values
    List,
}

#[derive(Subcommand)]
enum NoteCommands {
    /// List all notes
    List {
        /// Show notes in plain text mode (non-interactive)
        #[arg(long)]
        plain: bool,
    },
    /// Get a specific note by ID
    Get {
        /// The ID of the note to retrieve
        note_id: String,
        /// Open the note in your default browser
        #[arg(long)]
        open_in_browser: bool,
    },
    /// Create a new note
    Create {
        /// The object the note belongs to (e.g., "people")
        #[arg(long)]
        parent_object: String,
        /// The ID of the record the note is associated with
        #[arg(long)]
        parent_record_id: String,
        /// The title of the note
        #[arg(long)]
        title: String,
        /// The content of the note
        #[arg(long)]
        content: String,
        /// The format of the content ("plaintext" or "markdown")
        #[arg(long, default_value = "plaintext")]
        format: String,
        /// Open the note in your default browser after creating it
        #[arg(long)]
        open_in_browser: bool,
    },
    /// Delete a note by ID
    Delete {
        /// The ID of the note to delete
        note_id: String,
    },
}

fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("attio");
    path.push("config.json");
    path
}

fn read_config() -> Result<models::Config, Box<dyn Error>> {
    let config_path = get_config_path();
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        // Try to parse as new Config format
        if let Ok(config) = serde_json::from_str::<models::Config>(&content) {
            return Ok(config);
        }
        // Fallback: try old format (just token as string or in object)
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content)
            && let Some(token) = data["token"].as_str()
        {
            return Ok(models::Config::new(token.to_string()));
        }
    }
    Err("Config file not found".into())
}

fn write_config(config: &models::Config) -> Result<(), Box<dyn Error>> {
    let config_path = get_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(config_path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

fn get_token() -> Result<String, Box<dyn Error>> {
    // 1. Check config file first
    if let Ok(config) = read_config() {
        let token = config.token.trim();
        if !token.is_empty() {
            return Ok(token.to_string());
        }
    }

    // 2. Fallback to environment variable
    if let Ok(token) = env::var("ATTIO_API_TOKEN") {
        let token = token.trim();
        if !token.is_empty() {
            return Ok(token.to_string());
        }
    }

    Err("Not authenticated. Please run `attio auth <token>`.".into())
}

fn get_config() -> Result<models::Config, Box<dyn Error>> {
    read_config()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { token } => {
            let trimmed_token = token.trim().to_string();
            let config = if let Ok(mut existing_config) = read_config() {
                existing_config.token = trimmed_token;
                existing_config
            } else {
                models::Config::new(trimmed_token)
            };
            write_config(&config)?;
            println!(
                "✅ Successfully authenticated! Token saved to {:?}",
                get_config_path()
            );
        }
        Commands::Config { action } => match action {
            ConfigCommands::Set { key, value } => {
                let mut config = read_config().unwrap_or_else(|_| {
                    eprintln!("⚠️  No config found. Creating new config...");
                    models::Config::new(String::new())
                });

                match key.as_str() {
                    "cache-limit-mb" => {
                        let limit: u64 = value.parse().map_err(
                            |_| "Invalid value. cache-limit-mb must be a positive number.",
                        )?;
                        config.cache_limit_mb = limit;
                        write_config(&config)?;
                        println!("✅ Set cache-limit-mb to {}", limit);
                    }
                    _ => {
                        return Err(format!(
                            "Unknown config key: {}. Available keys: cache-limit-mb",
                            key
                        )
                        .into());
                    }
                }
            }
            ConfigCommands::Get { key } => {
                let config = get_config()?;
                match key.as_str() {
                    "cache-limit-mb" => {
                        println!("{}", config.cache_limit_mb);
                    }
                    _ => {
                        return Err(format!(
                            "Unknown config key: {}. Available keys: cache-limit-mb",
                            key
                        )
                        .into());
                    }
                }
            }
            ConfigCommands::List => {
                let config = get_config()?;
                let mut table = comfy_table::Table::new();
                table
                    .set_header(vec!["Key", "Value"])
                    .load_preset(comfy_table::presets::UTF8_HORIZONTAL_ONLY)
                    .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

                table.add_row(vec!["token", &config.token]);
                table.add_row(vec!["cache-limit-mb", &config.cache_limit_mb.to_string()]);

                println!("{table}");
            }
        },
        Commands::Notes { action } => {
            let token = get_token()?;
            let config = get_config().unwrap_or_else(|_| models::Config::new(token.clone()));
            let client = AttioClient::new(token);
            match action {
                NoteCommands::List { plain } => {
                    if plain {
                        let response = client.list_notes(None, None).await?;

                        let mut table = comfy_table::Table::new();
                        table
                            .set_header(vec!["#", "ID", "Title", "Content"])
                            .load_preset(comfy_table::presets::UTF8_HORIZONTAL_ONLY)
                            .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

                        for (i, note) in response.data.into_iter().enumerate() {
                            table.add_row(vec![
                                (i + 1).to_string(),
                                note.id.note_id,
                                note.title,
                                note.content_plaintext,
                            ]);
                        }

                        println!("{table}");
                    } else {
                        tui::run_list_tui(client, config.cache_limit_mb).await?;
                    }
                }
                NoteCommands::Get {
                    note_id,
                    open_in_browser,
                } => {
                    let response = client.get_note(&note_id).await?;
                    let note = response.data;

                    let mut table = comfy_table::Table::new();
                    table
                        .set_header(vec!["Attribute", "Value"])
                        .load_preset(comfy_table::presets::UTF8_HORIZONTAL_ONLY)
                        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

                    table.add_row(vec!["ID", &note.id.note_id]);
                    table.add_row(vec!["Title", &note.title]);
                    table.add_row(vec!["Content", &note.content_plaintext]);

                    println!("{table}");

                    if open_in_browser {
                        let id_response = client.identify().await?;
                        if let Some(slug) = id_response.workspace_slug {
                            // Map common plural objects to singular for the URL
                            let parent = match note.parent_object.as_str() {
                                "people" => "person",
                                "companies" => "company",
                                other => other,
                            };
                            let url = format!(
                                "https://app.attio.com/{}/{}/{}/notes?modal=note&id={}",
                                slug, parent, note.parent_record_id, note.id.note_id
                            );
                            println!("🔗 Opening note in browser...");
                            if let Err(e) = webbrowser::open(&url) {
                                eprintln!("Failed to open browser: {}", e);
                            }
                        } else {
                            println!(
                                "⚠️ Could not determine workspace slug to open identification URL."
                            );
                        }
                    }
                }
                NoteCommands::Create {
                    parent_object,
                    parent_record_id,
                    title,
                    content,
                    format,
                    open_in_browser,
                } => {
                    let request = crate::models::CreateNoteRequest {
                        data: crate::models::CreateNoteData {
                            parent_object,
                            parent_record_id,
                            title,
                            content,
                            format,
                        },
                    };
                    let response = client.create_note(request).await?;
                    let note = response.data;
                    println!("✅ Note created successfully!");

                    let mut table = comfy_table::Table::new();
                    table
                        .set_header(vec!["Attribute", "Value"])
                        .load_preset(comfy_table::presets::UTF8_HORIZONTAL_ONLY)
                        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

                    table.add_row(vec!["ID", &note.id.note_id]);
                    table.add_row(vec!["Title", &note.title]);
                    table.add_row(vec!["Content", &note.content_plaintext]);

                    println!("{table}");

                    if open_in_browser {
                        let id_response = client.identify().await?;
                        if let Some(slug) = id_response.workspace_slug {
                            let parent = match note.parent_object.as_str() {
                                "people" => "person",
                                "companies" => "company",
                                other => other,
                            };
                            let url = format!(
                                "https://app.attio.com/{}/{}/{}/notes?modal=note&id={}",
                                slug, parent, note.parent_record_id, note.id.note_id
                            );
                            println!("🔗 Opening note in browser...");
                            if let Err(e) = webbrowser::open(&url) {
                                eprintln!("Failed to open browser: {}", e);
                            }
                        }
                    }
                }
                NoteCommands::Delete { note_id } => {
                    client.delete_note(&note_id).await?;
                    println!("✅ Note {} deleted successfully.", note_id);
                }
            }
        }
    }

    Ok(())
}
