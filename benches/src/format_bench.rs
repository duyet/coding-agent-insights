//! Output formatting benchmarks
//!
//! Benchmarks for output formatting operations.

use divan::black_box;
use cai_core::{Entry, Source, Metadata};
use cai_output::{JsonFormatter, JsonlFormatter, Formatter};
use chrono::Utc;
use std::io::Cursor;

fn generate_entries(count: usize) -> Vec<Entry> {
    (0..count)
        .map(|i| Entry {
            id: format!("format-bench-{}", i),
            source: match i % 3 {
                0 => Source::Claude,
                1 => Source::Codex,
                _ => Source::Git,
            },
            timestamp: Utc::now() - chrono::Duration::seconds(i as i64),
            prompt: format!("Prompt {} with varying content length to test formatting performance realistically", i),
            response: format!("Response {} with content that includes code examples: ```rust\nfn test() {{}}\n``` and explanations", i),
            metadata: Metadata {
                file_path: Some(format!("src/module/file{}.rs", i)),
                repo_url: Some(format!("https://github.com/org/repo-{}/blob/main/file.rs", i % 5)),
                commit_hash: Some(format!("{:040x}", i)),
                language: Some(["Rust", "Python", "JavaScript", "TypeScript", "Go"][i % 5].to_string()),
                extra: {
                    let mut map = std::collections::HashMap::new();
                    map.insert("lines".to_string(), (i * 10).to_string());
                    map.insert("complexity".to_string(), ["low", "medium", "high"][i % 3].to_string());
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

    /// Benchmark JSON formatting (small result set)
    #[divan::bench(args = [10, 100, 1_000])]
    fn bench_json_format_small(bencher: divan::Bencher, count: usize) {
        let entries = generate_entries(count);

        bencher
            .counter(count)
            .with_inputs(|| entries.clone())
            .bench_values(|entries| {
                let mut buffer = Cursor::new(Vec::new());
                let formatter = JsonFormatter::default();
                formatter.format(&entries, &mut buffer).unwrap();
                black_box(buffer.into_inner());
            });
    }

    /// Benchmark JSON formatting (large result set)
    #[divan::bench(args = [10_000])]
    fn bench_json_format_large(bencher: divan::Bencher, count: usize) {
        let entries = generate_entries(count);

        bencher
            .counter(count)
            .with_inputs(|| entries.clone())
            .bench_values(|entries| {
                let mut buffer = Cursor::new(Vec::new());
                let formatter = JsonFormatter::default();
                formatter.format(&entries, &mut buffer).unwrap();
                black_box(buffer.into_inner());
            });
    }

    /// Benchmark JSONL formatting (small result set)
    #[divan::bench(args = [10, 100, 1_000])]
    fn bench_jsonl_format_small(bencher: divan::Bencher, count: usize) {
        let entries = generate_entries(count);

        bencher
            .counter(count)
            .with_inputs(|| entries.clone())
            .bench_values(|entries| {
                let mut buffer = Cursor::new(Vec::new());
                let formatter = JsonlFormatter::default();
                formatter.format(&entries, &mut buffer).unwrap();
                black_box(buffer.into_inner());
            });
    }

    /// Benchmark JSONL formatting (large result set)
    #[divan::bench(args = [10_000])]
    fn bench_jsonl_format_large(bencher: divan::Bencher, count: usize) {
        let entries = generate_entries(count);

        bencher
            .counter(count)
            .with_inputs(|| entries.clone())
            .bench_values(|entries| {
                let mut buffer = Cursor::new(Vec::new());
                let formatter = JsonlFormatter::default();
                formatter.format(&entries, &mut buffer).unwrap();
                black_box(buffer.into_inner());
            });
    }

    /// Benchmark JSON vs JSONL performance comparison
    #[divan::bench(args = [
        ("json", 100),
        ("jsonl", 100),
        ("json", 1_000),
        ("jsonl", 1_000),
        ("json", 10_000),
        ("jsonl", 10_000),
    ])]
    fn bench_format_comparison(bencher: divan::Bencher, (format_type, count): (&str, usize)) {
        let entries = generate_entries(count);

        bencher
            .counter(count)
            .with_inputs(|| entries.clone())
            .bench_values(|entries| {
                let mut buffer = Cursor::new(Vec::new());
                match format_type {
                    "json" => {
                        let formatter = JsonFormatter::default();
                        formatter.format(&entries, &mut buffer).unwrap();
                    }
                    "jsonl" => {
                        let formatter = JsonlFormatter::default();
                        formatter.format(&entries, &mut buffer).unwrap();
                    }
                    _ => panic!("Unknown format type"),
                }
                black_box(buffer.into_inner());
            });
    }

    /// Benchmark serialization overhead
    #[divan::bench(args = [1_000])]
    fn bench_serialization_only(bencher: divan::Bencher, count: usize) {
        let entries = generate_entries(count);

        bencher
            .counter(count)
            .with_inputs(|| entries.clone())
            .bench_values(|entries| {
                black_box(serde_json::to_string(&entries).unwrap());
            });
    }

    /// Benchmark formatting with empty result set
    #[divan::bench]
    fn bench_empty_result(bencher: divan::Bencher) {
        let entries: Vec<Entry> = vec![];

        bencher.bench(|| {
            let mut buffer = Cursor::new(Vec::new());
            let formatter = JsonFormatter::default();
            formatter.format(&entries, &mut buffer).unwrap();
            black_box(buffer.into_inner());
        });
    }

    /// Benchmark formatting with single entry
    #[divan::bench]
    fn bench_single_entry(bencher: divan::Bencher) {
        let entries = generate_entries(1);

        bencher.bench(|| {
            let mut buffer = Cursor::new(Vec::new());
            let formatter = JsonFormatter::default();
            formatter.format(&entries, &mut buffer).unwrap();
            black_box(buffer.into_inner());
        });
    }

    /// Benchmark output size comparison
    #[divan::bench(args = [
        ("json", 100),
        ("jsonl", 100),
        ("json", 1_000),
        ("jsonl", 1_000),
    ])]
    fn bench_output_size(bencher: divan::Bencher, (format_type, count): (&str, usize)) {
        let entries = generate_entries(count);

        bencher
            .counter(count)
            .with_inputs(|| entries.clone())
            .bench_values(|entries| {
                let mut buffer = Cursor::new(Vec::new());
                match format_type {
                    "json" => {
                        let formatter = JsonFormatter::default();
                        formatter.format(&entries, &mut buffer).unwrap();
                    }
                    "jsonl" => {
                        let formatter = JsonlFormatter::default();
                        formatter.format(&entries, &mut buffer).unwrap();
                    }
                    _ => panic!("Unknown format type"),
                }
                let output = buffer.into_inner();
                black_box(output.len());
            });
    }

    /// Benchmark formatting with complex metadata
    #[divan::bench(args = [1_000])]
    fn bench_complex_metadata(bencher: divan::Bencher, count: usize) {
        let entries: Vec<Entry> = (0..count)
            .map(|i| {
                let mut extra = std::collections::HashMap::new();
                for j in 0..10 {
                    extra.insert(format!("key_{}", j), format!("value_{}_with_longer_string_content", j));
                }
                Entry {
                    id: format!("complex-{}", i),
                    source: Source::Claude,
                    timestamp: Utc::now(),
                    prompt: "Complex prompt".to_string(),
                    response: "Complex response".to_string(),
                    metadata: Metadata {
                        file_path: Some(format!("path/{}]/file.rs", i)),
                        repo_url: Some(format!("https://github.com/org/repo{}", i)),
                        commit_hash: Some(format!("{:040x}", i)),
                        language: Some("Rust".to_string()),
                        extra,
                    },
                }
            })
            .collect();

        bencher
            .counter(count)
            .with_inputs(|| entries.clone())
            .bench_values(|entries| {
                let mut buffer = Cursor::new(Vec::new());
                let formatter = JsonFormatter::default();
                formatter.format(&entries, &mut buffer).unwrap();
                black_box(buffer.into_inner());
            });
    }
}
