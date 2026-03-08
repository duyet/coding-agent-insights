//! CAI CLI - Main command-line interface

#![warn(missing_docs, unused_crate_dependencies)]

use clap::{Parser, Subcommand};
use cai_ingest::{IngestConfig, Ingestor};
use colored::Colorize;
use std::path::PathBuf;

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
            println!("{} {}", "Querying:".green(), query);
            println!("{} {}", "Output:".cyan(), output);
            println!("\n{}", "[Data results would appear here]".dimmed());
            Ok(())
        }
        Commands::Ingest { source, path } => {
            execute_ingest(&source, path.as_deref()).await
        }
        Commands::Tui => {
            // Initialize in-memory storage with mock data for testing
            let storage = std::sync::Arc::new(cai_storage::MemoryStorage::with_mock_data());
            cai_tui::run(storage).await
        }
        #[cfg(feature = "web")]
        Commands::Web { port } => {
            let storage = std::sync::Arc::new(cai_storage::MemoryStorage::new());
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
