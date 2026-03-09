//! High-level ingestion orchestrator

use crate::{ClaudeParser, CodexParser, GitScanner};
use cai_core::Result;
use cai_storage::Storage;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Configuration for ingestion
#[derive(Debug, Clone)]
pub struct IngestConfig {
    /// Parse Claude Code conversations
    pub parse_claude: bool,
    /// Parse Codex CLI history
    pub parse_codex: bool,
    /// Scan Git repositories
    pub scan_git: bool,
    /// Git repository paths to scan
    pub git_repos: Vec<PathBuf>,
    /// Custom Claude conversations directory
    pub claude_dir: Option<PathBuf>,
    /// Custom Codex history file
    pub codex_file: Option<PathBuf>,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            parse_claude: true,
            parse_codex: true,
            scan_git: false,
            git_repos: Vec::new(),
            claude_dir: None,
            codex_file: None,
        }
    }
}

/// High-level ingestion orchestrator
pub struct Ingestor {
    config: IngestConfig,
}

impl Ingestor {
    /// Create a new ingestor with configuration
    pub fn new(config: IngestConfig) -> Self {
        Self { config }
    }

    /// Create ingestor with default configuration
    pub fn with_defaults() -> Self {
        Self::new(IngestConfig::default())
    }

    /// Ingest all configured sources and store entries
    pub async fn ingest_all<S: Storage>(&self, storage: &S) -> Result<usize> {
        let mut total_count = 0;

        if self.config.parse_claude {
            info!("Parsing Claude Code conversations");
            let claude_entries = self.parse_claude().await?;
            debug!("Found {} Claude entries", claude_entries.len());
            for entry in claude_entries {
                storage.store(&entry).await?;
                total_count += 1;
            }
        }

        if self.config.parse_codex {
            info!("Parsing Codex CLI history");
            let codex_entries = self.parse_codex().await?;
            debug!("Found {} Codex entries", codex_entries.len());
            for entry in codex_entries {
                storage.store(&entry).await?;
                total_count += 1;
            }
        }

        if self.config.scan_git {
            info!("Scanning Git repositories");
            for repo_path in &self.config.git_repos {
                let git_entries = self.scan_git(repo_path).await?;
                debug!(
                    "Found {} Git entries in {}",
                    git_entries.len(),
                    repo_path.display()
                );
                for entry in git_entries {
                    storage.store(&entry).await?;
                    total_count += 1;
                }
            }
        }

        Ok(total_count)
    }

    async fn parse_claude(&self) -> Result<Vec<cai_core::Entry>> {
        let parser = if let Some(ref dir) = self.config.claude_dir {
            ClaudeParser::new(dir)
        } else {
            ClaudeParser::with_default_path()
                .map_err(|e| cai_core::Error::Message(e.to_string()))?
        };

        parser
            .parse_all()
            .map_err(|e| cai_core::Error::Message(e.to_string()))
    }

    async fn parse_codex(&self) -> Result<Vec<cai_core::Entry>> {
        let parser = if let Some(ref file) = self.config.codex_file {
            CodexParser::new(file)
        } else {
            CodexParser::with_default_path().map_err(|e| cai_core::Error::Message(e.to_string()))?
        };

        parser
            .parse_all()
            .map_err(|e| cai_core::Error::Message(e.to_string()))
    }

    async fn scan_git(&self, repo_path: &Path) -> Result<Vec<cai_core::Entry>> {
        let scanner = GitScanner::new(repo_path);
        scanner
            .scan()
            .map_err(|e| cai_core::Error::Message(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cai_storage::MemoryStorage;

    #[tokio::test]
    async fn test_ingestor_empty_config() {
        let config = IngestConfig {
            parse_claude: false,
            parse_codex: false,
            scan_git: false,
            ..Default::default()
        };

        let ingestor = Ingestor::new(config);
        let storage = MemoryStorage::new();

        let count = ingestor.ingest_all(&storage).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_ingestor_with_git() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize git repo with a commit
        let repo = git2::Repository::init(repo_path).unwrap();
        let test_file = repo_path.join("test.txt");
        std::fs::write(&test_file, "test").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(std::path::Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = git2::Signature::now("Test", "test@test.com").unwrap();

        repo.commit(Some("HEAD"), &sig, &sig, "Test commit", &tree, &[])
            .unwrap();

        let config = IngestConfig {
            parse_claude: false,
            parse_codex: false,
            scan_git: true,
            git_repos: vec![repo_path.to_path_buf()],
            ..Default::default()
        };

        let ingestor = Ingestor::new(config);
        let storage = MemoryStorage::new();

        let count = ingestor.ingest_all(&storage).await.unwrap();
        assert_eq!(count, 1);

        let entries = storage.query(None).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert!(matches!(entries[0].source, cai_core::Source::Git));
    }
}
