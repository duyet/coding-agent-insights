//! CAI CLI - Main command-line interface

#![warn(missing_docs, unused_crate_dependencies)]

use clap::{Parser, Subcommand};
use colored::Colorize;

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
            println!("{} {}", "Ingesting from:".green(), source);
            if let Some(p) = path {
                println!("{} {}", "Path:".cyan(), p);
            }
            Ok(())
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
