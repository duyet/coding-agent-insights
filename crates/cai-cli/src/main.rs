//! CAI CLI - Main command-line interface
//!
//! Orchestrates all CAI subsystems and presents a polished terminal UI
//! with consistent styling, color theming, and user-friendly output.

#![warn(missing_docs, unused_crate_dependencies)]

mod config;

use cai_core::{Entry, Metadata, Source};
use cai_ingest::{IngestConfig, Ingestor};
use cai_output::{Formatter, StatsFormatter};
use cai_storage::Storage;
use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};
use colored::Colorize;
use config::load_config;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;

// ---------------------------------------------------------------------------
// Theme helpers
// ---------------------------------------------------------------------------

/// Check whether the user has opted out of color via env var.
fn color_disabled() -> bool {
    std::env::var("NO_COLOR")
        .ok()
        .and_then(|v| {
            if v.is_empty() { None } else { Some(true) }
        })
        .unwrap_or(false)
}

/// Render a structured error message to stderr
fn print_error(msg: &str) {
    if color_disabled() {
        eprintln!("error: {}", msg);
    } else {
        eprintln!("{} {}", "error".red().bold(), msg);
    }
}

// ---------------------------------------------------------------------------
// Mock data
// ---------------------------------------------------------------------------

/// Create storage with mock data for testing
async fn create_storage_with_mock_data() -> cai_storage::MemoryStorage {
    let storage = cai_storage::MemoryStorage::new();

    let mock_entries = vec![
        Entry {
            id: "1".to_string(),
            source: Source::Claude,
            timestamp: Utc::now() - Duration::hours(2),
            prompt: "Help me refactor this Rust function to be more idiomatic".to_string(),
            response: "Here's a more idiomatic version using iterators and pattern matching..."
                .to_string(),
            metadata: Metadata {
                file_path: Some("src/main.rs".to_string()),
                language: Some("Rust".to_string()),
                ..Default::default()
            },
        },
        Entry {
            id: "2".to_string(),
            source: Source::Claude,
            timestamp: Utc::now() - Duration::hours(4),
            prompt: "Write a unit test for this module".to_string(),
            response: "Here are comprehensive unit tests using rstest...".to_string(),
            metadata: Metadata {
                file_path: Some("src/storage.rs".to_string()),
                language: Some("Rust".to_string()),
                ..Default::default()
            },
        },
        Entry {
            id: "3".to_string(),
            source: Source::Git,
            timestamp: Utc::now() - Duration::days(1),
            prompt: "feat: add user authentication".to_string(),
            response: "Implemented OAuth2 flow with session management".to_string(),
            metadata: Metadata {
                commit_hash: Some("abc123def456".to_string()),
                ..Default::default()
            },
        },
        Entry {
            id: "4".to_string(),
            source: Source::Codex,
            timestamp: Utc::now() - Duration::days(2),
            prompt: "Generate a function to parse JSON".to_string(),
            response: "Here's a JSON parsing function using serde_json...".to_string(),
            metadata: Metadata {
                file_path: Some("src/parser.rs".to_string()),
                language: Some("Rust".to_string()),
                ..Default::default()
            },
        },
    ];

    for entry in mock_entries {
        if let Err(e) = storage.store(&entry).await {
            tracing::warn!("Failed to store mock entry {}: {}", entry.id, e);
        }
    }

    storage
}

/// Generic helper to format results using any formatter
fn format_with_formatter<F: Formatter>(
    results: &[Entry],
    formatter: F,
    format_name: &str,
) -> cai_core::Result<String> {
    let mut buffer = Vec::new();
    formatter.format(results, &mut buffer)?;
    String::from_utf8(buffer).map_err(|e| {
        cai_core::Error::Message(format!("Invalid UTF-8 in {} output: {}", format_name, e))
    })
}

/// Coding Agent Insights - Query AI coding history
#[derive(Parser, Clone)]
#[command(name = "cai")]
#[command(about = "Superior AI coding history analyzer", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// Execute a query
    Query {
        /// SQL-like query string
        query: String,
        /// Output format (json, jsonl, csv, table, ai, stats)
        #[arg(short, long, default_value = "table")]
        output: String,
    },
    /// Ingest data from sources
    Ingest {
        /// Source type (claude, codex, git)
        #[arg(short, long)]
        source: String,
        /// Source path
        #[arg(short, long)]
        path: Option<String>,
    },
    /// Show statistics about stored entries
    Stats,
    /// Show database schema information
    Schema {
        /// Table name to describe (optional)
        #[arg(short, long)]
        table: Option<String>,
    },
    /// Interactive terminal UI
    Tui,
    /// Start web server
    Web {
        #[arg(short, long, default_value = "3000")]
        port: u16,
    },
}

/// Execute data ingestion from specified source
async fn execute_ingest(source: &str, path: Option<&str>) -> cai_core::Result<()> {
    // Build config based on source
    let config = match source.to_lowercase().as_str() {
        "claude" => IngestConfig {
            parse_claude: true,
            parse_codex: false,
            scan_git: false,
            claude_dir: path.map(PathBuf::from),
            ..Default::default()
        },
        "codex" => IngestConfig {
            parse_claude: false,
            parse_codex: true,
            scan_git: false,
            codex_file: path.map(PathBuf::from),
            ..Default::default()
        },
        "all" => IngestConfig {
            parse_claude: true,
            parse_codex: true,
            scan_git: false,
            claude_dir: path.map(PathBuf::from),
            codex_file: path.map(PathBuf::from),
            ..Default::default()
        },
        _ => {
            print_error(&format!(
                "Unknown source '{}'. Valid options: claude, codex, all",
                source
            ));
            return Err(cai_core::Error::Message(format!(
                "Unknown source: '{}'. Valid options: claude, codex, all",
                source
            )));
        }
    };

    // Create ingestor and storage
    let ingestor = Ingestor::new(config);
    let storage = cai_storage::MemoryStorage::new();

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message(format!("Ingesting from {}...", source));

    // Execute ingestion
    let count = match ingestor.ingest_all(&storage).await {
        Ok(count) => count,
        Err(e) => {
            print_error(&e.to_string());
            std::process::exit(1);
        }
    };

    spinner.finish_and_clear();
    println!("{} {} entries", "✔ Ingestion complete:".green().bold(), count);
    Ok(())
}

/// Show statistics about stored entries
async fn execute_stats() -> cai_core::Result<()> {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Computing statistics...");

    let storage = cai_storage::MemoryStorage::new();
    let entries = match storage.query(None as Option<&cai_storage::Filter>).await {
        Ok(entries) => entries,
        Err(e) => {
            print_error(&e.to_string());
            std::process::exit(1);
        }
    };

    spinner.finish_and_clear();

    if entries.is_empty() {
        eprintln!("{}", "No entries found.".dimmed());
        return Ok(());
    }

    let formatter = StatsFormatter::default();
    let mut buffer = Vec::new();
    formatter.format(&entries, &mut buffer)?;
    let output = String::from_utf8(buffer)
        .map_err(|e| cai_core::Error::Message(format!("Invalid UTF-8 in stats output: {}", e)))?;

    println!("{}", output);
    Ok(())
}

/// Show database schema information
async fn execute_schema(table: Option<&str>) -> cai_core::Result<()> {
    eprintln!("{}", "▸ Schema".green().bold());

    if let Some(table_name) = table {
        if table_name.to_lowercase() != "entries" {
            return Err(cai_core::Error::Message(format!(
                "Unknown table: '{}'. Available tables: entries",
                table_name
            )));
        }

        eprintln!("  {} {}", "Table:".bold(), "entries".cyan());
        eprintln!(
            "{:<24} {:<20} {}",
            "Column".bold(),
            "Type".bold(),
            "Description".bold()
        );
        eprintln!("{}", "─".repeat(80).dimmed());
        eprintln!(
            "{:<24} {:<20} {}",
            "id".cyan(),
            "TEXT".yellow(),
            "Unique identifier"
        );
        eprintln!(
            "{:<24} {:<20} {}",
            "source".cyan(),
            "TEXT".yellow(),
            "Source system (Claude, Codex, Git, Other)"
        );
        eprintln!(
            "{:<24} {:<20} {}",
            "timestamp".cyan(),
            "TIMESTAMP".yellow(),
            "Interaction timestamp (UTC)"
        );
        eprintln!(
            "{:<24} {:<20} {}",
            "prompt".cyan(),
            "TEXT".yellow(),
            "User prompt/input"
        );
        eprintln!(
            "{:<24} {:<20} {}",
            "response".cyan(),
            "TEXT".yellow(),
            "AI response/output"
        );
        eprintln!(
            "{:<24} {:<20} {}",
            "metadata".cyan(),
            "JSON".yellow(),
            "Additional metadata (file_path, language, etc.)"
        );
    } else {
        eprintln!("  {} {}", "Available tables:".bold(), "entries".cyan());
        eprintln!();
        eprintln!("  {} `entries`", "Columns in".bold());
        eprintln!("    {} {}", "id       ".cyan(), "- Unique identifier (TEXT)".dimmed());
        eprintln!("    {} {}", "source   ".cyan(), "- Source system (TEXT)".dimmed());
        eprintln!("    {} {}", "timestamp".cyan(), "- Interaction timestamp (TIMESTAMP)".dimmed());
        eprintln!("    {} {}", "prompt   ".cyan(), "- User prompt/input (TEXT)".dimmed());
        eprintln!("    {} {}", "response ".cyan(), "- AI response/output (TEXT)".dimmed());
        eprintln!("    {} {}", "metadata ".cyan(), "- Additional metadata (JSON)".dimmed());
        eprintln!();
        eprintln!("  {}:", "Query examples".bold().green());
        eprintln!("    {}", "SELECT * FROM entries LIMIT 10".dimmed());
        eprintln!("    {}", "SELECT * FROM entries WHERE source = 'Claude'".dimmed());
    }

    Ok(())
}

/// Execute a SQL query and display results
async fn execute_query(query: &str, output_format: &str) -> cai_core::Result<()> {
    eprintln!("{} {}", "▸ Query:".green().bold(), query.dimmed());

    // TODO: Use persistent storage from config instead of mock data
    let storage = create_storage_with_mock_data().await;

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Executing query...");

    let query_engine = cai_query::QueryEngine::new(storage);
    let results = query_engine
        .execute(query)
        .await
        .map_err(|e| {
            print_error(&format!("Query execution failed: {}", e));
            cai_core::Error::Message(format!("Query execution failed: {}", e))
        })?;

    spinner.finish_and_clear();

    if results.is_empty() {
        eprintln!("{}", "∅ No results found.".dimmed());
        return Ok(());
    }

    eprintln!(
        "{} {} {}",
        "📋".dimmed(),
        format!("{} results", results.len()).cyan(),
        "•".dimmed()
    );

    let output = match output_format.to_lowercase().as_str() {
        "json" => format_with_formatter(&results, cai_output::JsonFormatter::default(), "json")?,
        "jsonl" => format_with_formatter(&results, cai_output::JsonlFormatter::default(), "jsonl")?,
        "csv" => format_with_formatter(&results, cai_output::CsvFormatter::default(), "csv")?,
        "table" => format_with_formatter(&results, cai_output::TableFormatter::default(), "table")?,
        "ai" => format_with_formatter(&results, cai_output::AiFormatter::default(), "ai")?,
        "stats" => format_with_formatter(&results, cai_output::StatsFormatter::default(), "stats")?,
        _ => {
            print_error(&format!(
                "Unknown format '{}'. Valid options: json, jsonl, csv, table, ai, stats",
                output_format
            ));
            return Err(cai_core::Error::Message(format!(
                "Unknown output format: '{}'. Valid options: json, jsonl, csv, table, ai, stats",
                output_format
            )));
        }
    };

    println!("{}", output);
    Ok(())
}

#[tokio::main]
async fn main() -> cai_core::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Load configuration
    let app_config = load_config();
    tracing::debug!(
        "Loaded config: storage type = {}",
        app_config.storage.r#type
    );

    let cli = Cli::parse();

    eprintln!(
        "{} {} {}",
        "cai".bold().cyan(),
        "v0.1.0".dimmed(),
        "— AI coding history analyzer".dimmed()
    );

    match cli.command {
        Commands::Query { query, output } => execute_query(&query, &output).await,
        Commands::Ingest { source, path } => execute_ingest(&source, path.as_deref()).await,
        Commands::Stats => execute_stats().await,
        Commands::Schema { table } => execute_schema(table.as_deref()).await,
        Commands::Tui => {
            let storage = Arc::new(create_storage_with_mock_data().await);
            cai_tui::run(storage).await
        }
        #[cfg(feature = "web")]
        Commands::Web { port } => {
            let web_config = cai_web::Config {
                port,
                host: "127.0.0.1".to_string(),
            };
            eprintln!("{} {}", "Starting web server on port:".green(), port);
            let storage = std::sync::Arc::new(cai_storage::MemoryStorage::new());
            cai_web::run(storage, web_config).await
        }
        #[cfg(not(feature = "web"))]
        Commands::Web { .. } => {
            print_error("Web feature not enabled. Build with '--features web'.");
            Err(cai_core::Error::Message(
                "Web feature not enabled".to_string(),
            ))
        }
    }
}
