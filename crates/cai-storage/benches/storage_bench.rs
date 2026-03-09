//! Benchmark tests for CAI storage

use cai_core::{Entry, Metadata, Source};
use cai_storage::{Filter, MemoryStorage, Storage};
use chrono::{Duration, Utc};
use std::collections::HashMap;

fn create_benchmark_entry(id: usize) -> Entry {
    let mut extra = HashMap::new();
    extra.insert("complexity".to_string(), format!("{}", id % 10));
    extra.insert("lines".to_string(), format!("{}", id * 10));

    Entry {
        id: format!("bench-entry-{}", id),
        source: match id % 4 {
            0 => Source::Claude,
            1 => Source::Codex,
            2 => Source::Git,
            _ => Source::Other("custom".to_string()),
        },
        timestamp: Utc::now() - Duration::seconds(id as i64),
        prompt: format!("Benchmark prompt {}", id),
        response: format!("Benchmark response {}", id),
        metadata: Metadata {
            file_path: Some(format!("src/file{}.rs", id % 100)),
            repo_url: Some("https://github.com/test/repo".to_string()),
            commit_hash: Some(format!("commit{}", id)),
            language: Some("Rust".to_string()),
            extra,
        },
    }
}

fn main() {
    divan::main();
}

#[divan::bench(sample_count = 100, sample_size = 10)]
fn bench_store_single_entry(b: divan::Bencher) {
    let storage = MemoryStorage::new();
    let entry = create_benchmark_entry(1);

    b.bench(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async { storage.store(&entry).await }).unwrap()
    });
}

#[divan::bench(sample_count = 50, sample_size = 10)]
fn bench_store_100_entries(b: divan::Bencher) {
    b.bench_local(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let storage = MemoryStorage::new();
            for i in 0..100 {
                let entry = create_benchmark_entry(i);
                storage.store(&entry).await.unwrap();
            }
        });
    });
}

#[divan::bench(sample_count = 50, sample_size = 10)]
fn bench_store_1000_entries(b: divan::Bencher) {
    b.bench_local(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let storage = MemoryStorage::new();
            for i in 0..1000 {
                let entry = create_benchmark_entry(i);
                storage.store(&entry).await.unwrap();
            }
        });
    });
}

#[divan::bench(sample_count = 100, sample_size = 10)]
fn bench_get_by_id(b: divan::Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let storage = rt.block_on(async {
        let storage = MemoryStorage::new();
        for i in 0..100 {
            let entry = create_benchmark_entry(i);
            storage.store(&entry).await.unwrap();
        }
        storage
    });

    b.bench(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _ = rt.block_on(async { storage.get("bench-entry-50").await });
    });
}

#[divan::bench(sample_count = 100, sample_size = 10)]
fn bench_query_all_small(b: divan::Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let storage = rt.block_on(async {
        let storage = MemoryStorage::new();
        for i in 0..10 {
            let entry = create_benchmark_entry(i);
            storage.store(&entry).await.unwrap();
        }
        storage
    });

    b.bench(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _ = rt.block_on(async { storage.query(None).await });
    });
}

#[divan::bench(sample_count = 50, sample_size = 10)]
fn bench_query_all_large(b: divan::Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let storage = rt.block_on(async {
        let storage = MemoryStorage::new();
        for i in 0..1000 {
            let entry = create_benchmark_entry(i);
            storage.store(&entry).await.unwrap();
        }
        storage
    });

    b.bench(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _ = rt.block_on(async { storage.query(None).await });
    });
}

#[divan::bench(sample_count = 50, sample_size = 10)]
fn bench_query_with_filter(b: divan::Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let storage = rt.block_on(async {
        let storage = MemoryStorage::new();
        for i in 0..1000 {
            let entry = create_benchmark_entry(i);
            storage.store(&entry).await.unwrap();
        }
        storage
    });

    let filter = Filter {
        source: Some("Claude".to_string()),
        after: None,
        before: None,
    };

    b.bench(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _ = rt.block_on(async { storage.query(Some(&filter)).await });
    });
}

#[divan::bench(sample_count = 100, sample_size = 10)]
fn bench_count(b: divan::Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let storage = rt.block_on(async {
        let storage = MemoryStorage::new();
        for i in 0..100 {
            let entry = create_benchmark_entry(i);
            storage.store(&entry).await.unwrap();
        }
        storage
    });

    b.bench(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _ = rt.block_on(async { storage.count().await });
    });
}

#[divan::bench(sample_count = 50, sample_size = 10)]
fn bench_concurrent_stores(b: divan::Bencher) {
    b.bench_local(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let storage = std::sync::Arc::new(MemoryStorage::new());
            let mut handles = Vec::new();

            for i in 0..100 {
                let storage_clone = storage.clone();
                let handle = tokio::spawn(async move {
                    let entry = create_benchmark_entry(i);
                    storage_clone.store(&entry).await
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.await.unwrap().unwrap();
            }
        });
    });
}

#[divan::bench(sample_count = 50, sample_size = 10)]
fn bench_concurrent_queries(b: divan::Bencher) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let storage = rt.block_on(async {
        let storage = std::sync::Arc::new(MemoryStorage::new());
        for i in 0..1000 {
            let entry = create_benchmark_entry(i);
            storage.store(&entry).await.unwrap();
        }
        storage
    });

    b.bench_local(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut handles = Vec::new();

            for _ in 0..10 {
                let storage_clone = storage.clone();
                let handle = tokio::spawn(async move { storage_clone.query(None).await });
                handles.push(handle);
            }

            for handle in handles {
                handle.await.unwrap().unwrap();
            }
        });
    });
}
