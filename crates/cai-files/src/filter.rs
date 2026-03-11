//! In-memory filtering for loaded entries

use cai_core::Entry;
use std::collections::HashSet;

/// In-memory filter operations
pub struct FileFilterOps;

impl FileFilterOps {
    /// Filter entries by source
    pub fn by_source(entries: Vec<Entry>, sources: &[&str]) -> Vec<Entry> {
        if sources.is_empty() {
            return entries;
        }

        let source_set: HashSet<_> = sources.iter().map(|s| s.to_string()).collect();
        entries
            .into_iter()
            .filter(|e| source_set.contains(&format!("{:?}", e.source)))
            .collect()
    }

    /// Filter entries by date range
    pub fn by_date_range(
        entries: Vec<Entry>,
        after: Option<chrono::DateTime<chrono::Utc>>,
        before: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Vec<Entry> {
        entries
            .into_iter()
            .filter(|e| {
                if let Some(after) = after {
                    if e.timestamp < after {
                        return false;
                    }
                }
                if let Some(before) = before {
                    if e.timestamp > before {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// Filter entries by text search (simple)
    pub fn by_text(entries: Vec<Entry>, query: &str) -> Vec<Entry> {
        if query.is_empty() {
            return entries;
        }

        let query_lower = query.to_lowercase();
        entries
            .into_iter()
            .filter(|e| {
                e.prompt.to_lowercase().contains(&query_lower)
                    || e.response.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cai_core::{Metadata, Source};
    use chrono::Utc;

    #[test]
    fn test_filter_by_source() {
        let entries = vec![
            Entry {
                id: "1".to_string(),
                source: Source::Claude,
                timestamp: Utc::now(),
                prompt: "test".to_string(),
                response: "response".to_string(),
                metadata: Metadata::default(),
            },
            Entry {
                id: "2".to_string(),
                source: Source::Codex,
                timestamp: Utc::now(),
                prompt: "test".to_string(),
                response: "response".to_string(),
                metadata: Metadata::default(),
            },
        ];

        let filtered = FileFilterOps::by_source(entries, &["Claude"]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].source, Source::Claude);
    }

    #[test]
    fn test_filter_by_text() {
        let entries = vec![
            Entry {
                id: "1".to_string(),
                source: Source::Claude,
                timestamp: Utc::now(),
                prompt: "write rust code".to_string(),
                response: "here is rust code".to_string(),
                metadata: Metadata::default(),
            },
            Entry {
                id: "2".to_string(),
                source: Source::Claude,
                timestamp: Utc::now(),
                prompt: "write python code".to_string(),
                response: "here is python code".to_string(),
                metadata: Metadata::default(),
            },
        ];

        let filtered = FileFilterOps::by_text(entries, "rust");
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].prompt.contains("rust"));
    }
}
