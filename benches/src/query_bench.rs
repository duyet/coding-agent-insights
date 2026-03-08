//! Query performance benchmarks
//!
//! Benchmarks for query operations on large datasets.

use divan::black_box;
use cai_core::{Entry, Source, Metadata};
use cai_storage::{MemoryStorage, Storage, Filter};
use chrono::Utc;

fn generate_entries(count: usize) -> Vec<Entry> {
    (0..count)
        .map(|i| Entry {
            id: format!("bench-entry-{}", i),
            source: match i % 4 {
                0 => Source::Claude,
                1 => Source::Codex,
                2 => Source::Git,
                _ => Source::Other(format!("source-{}", i % 10)),
            },
            timestamp: Utc::now() - chrono::Duration::seconds(i as i64),
            prompt: format!("Benchmark prompt number {} with some text to make it realistic", i),
            response: format!("Benchmark response number {} with content that represents a typical AI response", i),
            metadata: Metadata {
                file_path: Some(format!("src/file{}.rs", i % 100)),
                repo_url: Some(format!("https://github.com/repo-{}", i % 10)),
                commit_hash: Some(format!("commit-{:040x}", i)),
                language: Some(if i % 3 == 0 { "Rust" } else if i % 3 == 1 { "Python" } else { "JavaScript" }.to_string()),
                extra: {
                    let mut map = std::collections::HashMap::new();
                    if i % 2 == 0 {
                        map.insert("complexity".to_string(), "high".to_string());
                    }
                    map.insert("index".to_string(), i.to_string());
                    map
                },
            },
        })
        .collect()
}

fn main() {
    divan::main();
}

mod benchmarks {
    use super::*;
    use std::sync::Arc;

    fn setup_storage(count: usize) -> Arc<MemoryStorage> {
        let storage = Arc::new(MemoryStorage::new());
        let rt = tokio::runtime::Runtime::new().unwrap();
        let entries = generate_entries(count);

        rt.block_on(async {
            for entry in &entries {
                storage.store(entry).await.unwrap();
            }
        });
        storage
    }

    /// Benchmark storing entries (small dataset)
    #[divan::bench(args = [10, 100, 1_000])]
    fn bench_store_small(bencher: divan::Bencher, count: usize) {
        let entries = generate_entries(count);

        bencher
            .with_inputs(|| entries.clone())
            .bench_values(|entries| {
                let storage = MemoryStorage::new();
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    for entry in &entries {
                        storage.store(entry).await.unwrap();
                    }
                });
            });
    }

    /// Benchmark storing entries (large dataset)
    #[divan::bench(args = [10_000])]
    fn bench_store_large(bencher: divan::Bencher, count: usize) {
        let entries = generate_entries(count);

        bencher
            .with_inputs(|| entries.clone())
            .bench_values(|entries| {
                let storage = MemoryStorage::new();
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    for entry in &entries {
                        storage.store(entry).await.unwrap();
                    }
                });
            });
    }

    /// Benchmark query all entries (small dataset)
    #[divan::bench(args = [10, 100, 1_000])]
    fn bench_query_all_small(bencher: divan::Bencher, count: usize) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = setup_storage(count);

        bencher.counter(count).bench(|| {
            rt.block_on(async {
                black_box(storage.query(None).await.unwrap())
            })
        });
    }

    /// Benchmark query all entries (large dataset)
    #[divan::bench(args = [10_000])]
    fn bench_query_all_large(bencher: divan::Bencher, count: usize) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = setup_storage(count);

        bencher.counter(count).bench(|| {
            rt.block_on(async {
                black_box(storage.query(None).await.unwrap())
            })
        });
    }

    /// Benchmark query by source filter
    #[divan::bench(args = [1_000, 10_000])]
    fn bench_query_by_source(bencher: divan::Bencher, count: usize) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = setup_storage(count);

        let filter = Filter {
            source: Some("Claude".to_string()),
            after: None,
            before: None,
        };

        bencher.counter(count).bench(|| {
            rt.block_on(async {
                black_box(storage.query(Some(&filter)).await.unwrap())
            })
        });
    }

    /// Benchmark query with date range filter
    #[divan::bench(args = [1_000, 10_000])]
    fn bench_query_by_date_range(bencher: divan::Bencher, count: usize) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = setup_storage(count);

        let filter = Filter {
            source: None,
            after: Some(Utc::now() - chrono::Duration::seconds((count / 2) as i64)),
            before: None,
        };

        bencher.counter(count).bench(|| {
            rt.block_on(async {
                black_box(storage.query(Some(&filter)).await.unwrap())
            })
        });
    }

    /// Benchmark query with combined filters
    #[divan::bench(args = [1_000, 10_000])]
    fn bench_query_combined_filters(bencher: divan::Bencher, count: usize) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = setup_storage(count);

        let filter = Filter {
            source: Some("Claude".to_string()),
            after: Some(Utc::now() - chrono::Duration::seconds((count / 2) as i64)),
            before: Some(Utc::now()),
        };

        bencher.counter(count).bench(|| {
            rt.block_on(async {
                black_box(storage.query(Some(&filter)).await.unwrap())
            })
        });
    }

    /// Benchmark get by ID
    #[divan::bench(args = [1_000, 10_000])]
    fn bench_get_by_id(bencher: divan::Bencher, count: usize) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = setup_storage(count);

        bencher.bench(|| {
            rt.block_on(async {
                black_box(storage.get("bench-entry-42").await.unwrap())
            })
        });
    }

    /// Benchmark count operation
    #[divan::bench(args = [1_000, 10_000])]
    fn bench_count(bencher: divan::Bencher, count: usize) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = setup_storage(count);

        bencher.counter(count).bench(|| {
            rt.block_on(async {
                black_box(storage.count().await.unwrap())
            })
        });
    }

    /// Benchmark concurrent queries
    #[divan::bench(args = [1_000])]
    fn bench_concurrent_queries(bencher: divan::Bencher, count: usize) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let storage = setup_storage(count);

        bencher.bench(|| {
            rt.block_on(async {
                let mut handles = Vec::new();
                for _ in 0..10 {
                    let storage = storage.clone();
                    handles.push(tokio::spawn(async move {
                        storage.query(None).await.unwrap()
                    }));
                }
                for handle in handles {
                    handle.await.unwrap();
                }
            })
        });
    }
}
