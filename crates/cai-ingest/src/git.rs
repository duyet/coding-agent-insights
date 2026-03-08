//! Git repository scanner

use crate::error::IngestError;
use cai_core::{Entry, Metadata, Source};
use chrono::{DateTime, Utc};
use git2::{Repository, Time};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::debug;

/// Scanner for Git repository commits
pub struct GitScanner {
    /// Repository path
    repo_path: PathBuf,
}

impl GitScanner {
    /// Create a new Git scanner
    ///
    /// # Arguments
    /// * `repo_path` - Path to the Git repository
    pub fn new<P: AsRef<Path>>(repo_path: P) -> Self {
        Self {
            repo_path: repo_path.as_ref().to_path_buf(),
        }
    }

    /// Scan repository and convert commits to entries
    pub fn scan(&self) -> Result<Vec<Entry>, IngestError> {
        let repo = Repository::open(&self.repo_path)
            .map_err(IngestError::GitError)?;

        // Get repo URL if available
        let repo_url = get_remote_url(&repo);

        let mut revwalk = repo.revwalk()
            .map_err(IngestError::GitError)?;

        // Try to push HEAD - this will fail on empty repositories
        match revwalk.push_head() {
            Ok(_) => {},
            Err(_) => {
                // Empty repository or unborn HEAD
                return Err(IngestError::NoFilesFound(self.repo_path.display().to_string()));
            }
        }

        let mut entries = Vec::new();

        for oid in revwalk {
            let oid = oid.map_err(IngestError::GitError)?;
            let commit = repo.find_commit(oid)
                .map_err(IngestError::GitError)?;

            debug!("Scanning commit: {}", commit.id());

            let entry = self.commit_to_entry(&commit, &repo_url)?;
            entries.push(entry);
        }

        if entries.is_empty() {
            return Err(IngestError::NoFilesFound(self.repo_path.display().to_string()));
        }

        Ok(entries)
    }

    /// Scan only commits since a given date
    pub fn scan_since(&self, since: DateTime<Utc>) -> Result<Vec<Entry>, IngestError> {
        let all_entries = self.scan()?;
        Ok(all_entries
            .into_iter()
            .filter(|e| e.timestamp >= since)
            .collect())
    }

    fn commit_to_entry(
        &self,
        commit: &git2::Commit,
        repo_url: &Option<String>,
    ) -> Result<Entry, IngestError> {
        let id = format!("git-{}", commit.id());
        let timestamp = git_time_to_datetime(commit.time());

        // Build prompt from commit message (first line) and author
        let prompt = format!(
            "{}\n\nAuthor: {} <{}>",
            commit.summary().unwrap_or(""),
            commit.author().name().unwrap_or(""),
            commit.author().email().unwrap_or("")
        );

        // Use full message as response
        let response = commit.message().unwrap_or("").to_string();

        let mut extra = HashMap::new();
        extra.insert("author_name".to_string(), commit.author().name().unwrap_or("").to_string());
        extra.insert("author_email".to_string(), commit.author().email().unwrap_or("").to_string());
        extra.insert("committer_name".to_string(), commit.committer().name().unwrap_or("").to_string());
        extra.insert("committer_email".to_string(), commit.committer().email().unwrap_or("").to_string());

        // Add parent commit IDs
        if let Ok(parent_id) = commit.parent_id(0) {
            extra.insert("parent_commit".to_string(), parent_id.to_string());
        }

        Ok(Entry {
            id,
            source: Source::Git,
            timestamp,
            prompt,
            response,
            metadata: Metadata {
                file_path: None,
                repo_url: repo_url.clone(),
                commit_hash: Some(commit.id().to_string()),
                language: None,
                extra,
            },
        })
    }
}

/// Get the remote URL of a repository
fn get_remote_url(repo: &Repository) -> Option<String> {
    repo.find_remote("origin")
        .ok()
        .and_then(|r| r.url().map(|u| u.to_string()))
}

/// Convert git2 Time to DateTime<Utc>
fn git_time_to_datetime(time: Time) -> DateTime<Utc> {
    DateTime::from_timestamp(time.seconds(), 0)
        .unwrap_or_else(Utc::now)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scan_git_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize a git repo
        let repo = Repository::init(repo_path).unwrap();

        // Create a test file and commit
        let test_file = repo_path.join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();

        let sig = git2::Signature::now("Test User", "test@example.com").unwrap();
        let oid = repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Test commit message\n\nThis is a test commit with more details.",
            &tree,
            &[],
        ).unwrap();

        // Scan the repo
        let scanner = GitScanner::new(repo_path);
        let entries = scanner.scan().unwrap();

        assert_eq!(entries.len(), 1);
        let entry = &entries[0];
        assert!(entry.id.starts_with("git-"));
        assert_eq!(entry.source, Source::Git);
        assert!(entry.prompt.contains("Test commit message"));
        assert!(entry.response.contains("more details"));
        assert_eq!(entry.metadata.commit_hash, Some(oid.to_string()));
    }

    #[test]
    fn test_scan_empty_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize empty git repo (no commits)
        Repository::init(repo_path).unwrap();

        let scanner = GitScanner::new(repo_path);
        let result = scanner.scan();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), IngestError::NoFilesFound(_)));
    }
}
