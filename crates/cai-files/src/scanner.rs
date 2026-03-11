//! File scanner - find conversation files on disk

use crate::Result;
use cai_core::Source;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Scan for conversation files on disk
pub struct FileScanner {
    claude_path: PathBuf,
    codex_path: PathBuf,
}

impl FileScanner {
    /// Create new scanner with default paths
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("Unable to determine home directory");

        Self {
            claude_path: home.join(".claude/conversations"),
            codex_path: home.join(".codex"),
        }
    }

    /// Create scanner with custom paths
    pub fn with_paths(claude_path: PathBuf, codex_path: PathBuf) -> Self {
        Self {
            claude_path,
            codex_path,
        }
    }

    /// Scan all configured paths for conversation files
    pub fn scan(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        // Scan Claude conversations
        if self.claude_path.exists() {
            debug!("Scanning Claude conversations: {:?}", self.claude_path);
            files.extend(self.scan_directory(&self.claude_path)?);
        }

        // Scan Codex history
        if self.codex_path.exists() {
            debug!("Scanning Codex history: {:?}", self.codex_path);
            files.extend(self.scan_codex()?);
        }

        Ok(files)
    }

    /// Find files matching specific filter
    pub fn find(&self, filter: &ScanFilter) -> Result<Vec<PathBuf>> {
        let all_files = self.scan()?;
        Ok(all_files
            .into_iter()
            .filter(|path| self.matches_filter(path, filter))
            .collect())
    }

    /// Scan directory for JSON files
    fn scan_directory(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let entries = std::fs::read_dir(dir)
            .map_err(|e| crate::Error::Message(format!("Failed to read directory {:?}: {}", dir, e)))?;

        let mut files = Vec::new();
        for entry in entries {
            let entry = entry
                .map_err(|e| crate::Error::Message(format!("Failed to read entry: {}", e)))?;
            let path = entry.path();

            // Only include JSON files
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                files.push(path);
            }
        }

        Ok(files)
    }

    /// Scan Codex JSONL file
    fn scan_codex(&self) -> Result<Vec<PathBuf>> {
        let jsonl_path = self.codex_path.join("history.jsonl");
        if jsonl_path.exists() {
            Ok(vec![jsonl_path])
        } else {
            Ok(Vec::new())
        }
    }

    /// Check if file matches filter criteria
    fn matches_filter(&self, path: &Path, filter: &ScanFilter) -> bool {
        // Source filter
        if let Some(ref source) = filter.source {
            let path_str = path.to_string_lossy();
            match source {
                Source::Claude => {
                    if !path_str.contains("claude") {
                        return false;
                    }
                }
                Source::Codex => {
                    if !path_str.contains("codex") {
                        return false;
                    }
                }
                _ => {}
            }
        }

        // Date filters would require loading metadata from file
        // For now, skip date filtering at scan level

        true
    }
}

impl Default for FileScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Filter for finding specific files
#[derive(Debug, Clone, Default)]
pub struct ScanFilter {
    /// Source to filter by
    pub source: Option<Source>,
    /// Minimum timestamp (requires loading file metadata)
    pub after: Option<chrono::DateTime<chrono::Utc>>,
    /// Maximum timestamp (requires loading file metadata)
    pub before: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_creation() {
        let scanner = FileScanner::new();
        assert!(scanner.claude_path.ends_with(".claude/conversations"));
        assert!(scanner.codex_path.ends_with(".codex"));
    }

    #[test]
    fn test_scanner_default() {
        let scanner = FileScanner::default();
        assert!(scanner.claude_path.ends_with(".claude/conversations"));
    }
}
