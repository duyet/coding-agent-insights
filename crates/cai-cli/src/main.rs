//! CAI CLI - Main command-line interface

#![warn(missing_docs, unused_crate_dependencies)]

use clap::{Parser, Subcommand};
use cai_core::{Entry, Metadata, Source};
use cai_ingest::{IngestConfig, Ingestor};
use cai_output::Formatter;
use cai_storage::Storage;
use chrono::{Duration, Utc};
use colored::Colorize;
use std::path::PathBuf;
use std::sync::Arc;

/// Create storage with mock data for testing
async fn create_storage_with_mock_data() -> cai_storage::MemoryStorage {
    let storage = cai_storage::MemoryStorage::new();

    let mock_entries = vec![
        Entry {
            id: "1".to_string(),
            source: Source::Claude,
            timestamp: Utc::now() - Duration::hours(2),
            prompt: "Help me refactor this Rust function to be more idiomatic".to_string(),
            response: "Here's a more idiomatic version using iterators and pattern matching...".to_string(),
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
        let _ = storage.store(&entry).await;
    }

    storage
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
        /// Output format
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
    println!("{} {}", "Ingesting from:".green(), source);

    // Build config based on source
    let config = match source.to_lowercase().as_str() {
        "claude" => IngestConfig {
            parse_claude: true,
            parse_codex: false,
            scan_git: false,
            claude_dir: path.map(|p| PathBuf::from(p)),
            ..Default::default()
        },
        "codex" => IngestConfig {
            parse_claude: false,
            parse_codex: true,
            scan_git: false,
            codex_file: path.map(|p| PathBuf::from(p)),
            ..Default::default()
        },
        "all" => IngestConfig {
            parse_claude: true,
            parse_codex: true,
            scan_git: false,
            claude_dir: path.map(|p| PathBuf::from(p)),
            codex_file: path.map(|p| PathBuf::from(p)),
            ..Default::default()
        },
        _ => {
            return Err(cai_core::Error::Message(format!(
                "Unknown source: '{}'. Valid options: claude, codex, all",
                source
            )));
        }
    };

    // Create ingestor and storage
    let ingestor = Ingestor::new(config);
    let storage = cai_storage::MemoryStorage::new();

    // Execute ingestion
    let count = match ingestor.ingest_all(&storage).await {
        Ok(count) => count,
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            std::process::exit(1);
        }
    };

    println!("\n{} {} entries", "Successfully ingested:".green(), count);
    Ok(())
}

/// Execute a SQL query and display results
async fn execute_query(query: &str, output_format: &str) -> cai_core::Result<()> {
    println!("{} {}", "Executing query:".green(), query.dimmed());

    // Initialize storage with mock data for now
    let storage = create_storage_with_mock_data().await;

    // Parse and execute query
    let query_engine = cai_query::QueryEngine::new(storage);
    let results = match query_engine.execute(query).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{} {}", "Error:".red(), e);
            std::process::exit(1);
        }
    };

    // Display results count
    println!("\n{} {} results", "Found:".cyan(), results.len());

    if results.is_empty() {
        println!("\n{}", "No results found.".dimmed());
        return Ok(());
    }

    // Format and display output
    let output = match output_format.to_lowercase().as_str() {
        "json" => format_as_json(&results)?,
        "jsonl" => format_as_jsonl(&results)?,
        "csv" => format_as_csv(&results)?,
        "table" => format_as_table(&results)?,
        "ai" => format_as_ai(&results)?,
        "stats" => format_as_stats(&results)?,
        _ => {
            return Err(cai_core::Error::Message(format!(
                "Unknown output format: '{}'. Valid options: json, jsonl, csv, table, ai, stats",
                output_format
            )));
        }
    };

    println!("\n{}", output);
    Ok(())
}

/// Format results as JSON
fn format_as_json(results: &[cai_core::Entry]) -> cai_core::Result<String> {
    let formatter = cai_output::JsonFormatter::default();
    let mut buffer = Vec::new();
    formatter.format(results, &mut buffer)?;
    String::from_utf8(buffer)
        .map_err(|e| cai_core::Error::Message(format!("Invalid UTF-8 in JSON output: {}", e)))
}

/// Format results as JSON Lines
fn format_as_jsonl(results: &[cai_core::Entry]) -> cai_core::Result<String> {
    let formatter = cai_output::JsonlFormatter::default();
    let mut buffer = Vec::new();
    formatter.format(results, &mut buffer)?;
    String::from_utf8(buffer)
        .map_err(|e| cai_core::Error::Message(format!("Invalid UTF-8 in JSONL output: {}", e)))
}

/// Format results as CSV
fn format_as_csv(results: &[cai_core::Entry]) -> cai_core::Result<String> {
    let formatter = cai_output::CsvFormatter::default();
    let mut buffer = Vec::new();
    formatter.format(results, &mut buffer)?;
    String::from_utf8(buffer)
        .map_err(|e| cai_core::Error::Message(format!("Invalid UTF-8 in CSV output: {}", e)))
}

/// Format results as table
fn format_as_table(results: &[cai_core::Entry]) -> cai_core::Result<String> {
    let formatter = cai_output::TableFormatter::default();
    let mut buffer = Vec::new();
    formatter.format(results, &mut buffer)?;
    String::from_utf8(buffer)
        .map_err(|e| cai_core::Error::Message(format!("Invalid UTF-8 in table output: {}", e)))
}

/// Format results as AI-optimized
fn format_as_ai(results: &[cai_core::Entry]) -> cai_core::Result<String> {
    let formatter = cai_output::AiFormatter::default();
    let mut buffer = Vec::new();
    formatter.format(results, &mut buffer)?;
    String::from_utf8(buffer)
        .map_err(|e| cai_core::Error::Message(format!("Invalid UTF-8 in AI output: {}", e)))
}

/// Format results as statistics
fn format_as_stats(results: &[cai_core::Entry]) -> cai_core::Result<String> {
    let formatter = cai_output::StatsFormatter::default();
    let mut buffer = Vec::new();
    formatter.format(results, &mut buffer)?;
    String::from_utf8(buffer)
        .map_err(|e| cai_core::Error::Message(format!("Invalid UTF-8 in stats output: {}", e)))
}

#[tokio::main]
async fn main() -> cai_core::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Query { query, output } => {
            execute_query(&query, &output).await
        }
        Commands::Ingest { source, path } => {
            execute_ingest(&source, path.as_deref()).await
        }
        Commands::Tui => {
            // Initialize in-memory storage with mock data for testing
            let storage = Arc::new(create_storage_with_mock_data().await);
            cai_tui::run(storage).await
        }
        #[cfg(feature = "web")]
        Commands::Web { port } => {
            let storage = Arc::new(cai_storage::MemoryStorage::new());
            let config = cai_web::Config {
                port,
                host: "127.0.0.1".to_string(),
            };
            println!("{} {}", "Starting web server on port:".green(), port);
            cai_web::run(storage, config).await
        }
        #[cfg(not(feature = "web"))]
        Commands::Web { .. } => {
            eprintln!("{}", "Web feature not enabled. Build with --features web.".red());
            std::process::exit(1);
        }
    }
}
