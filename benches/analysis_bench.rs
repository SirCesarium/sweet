// @swt-disable max-repetition
//! Benchmarks for the Sweet code analysis engine.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::collections::HashSet;
use std::path::PathBuf;
use swt::Config;
use swt::analyzer::analyze_content;

/// Benchmark for analyzing a standard-sized Rust file.
fn bench_standard_analysis(c: &mut Criterion) {
    let config = Config::default();
    let extension = "rs";
    let thresholds = config.get_thresholds(extension);
    let path = PathBuf::from("src/lib.rs");
    let content = "use std::io;\nfn main() {\n    println!(\"hello world\");\n}\n".repeat(100);
    let disabled_rules = HashSet::new();

    c.bench_function("analyze_content_standard", |b| {
        b.iter(|| {
            analyze_content(
                black_box(content.as_bytes()),
                black_box(extension),
                black_box(&thresholds),
                black_box(&path),
                black_box(&config),
                black_box(&disabled_rules),
                black_box(true),
            )
        });
    });
}

/// Benchmark for analyzing a large file with many duplications.
fn bench_repetition_heavy(c: &mut Criterion) {
    let config = Config::default();
    let extension = "rs";
    let thresholds = config.get_thresholds(extension);
    let path = PathBuf::from("heavy.rs");
    let content =
        "fn duplicate() {\n    let x = 1;\n    let y = 2;\n    let z = x + y;\n}\n".repeat(500);
    let disabled_rules = HashSet::new();

    c.bench_function("analyze_content_repetition", |b| {
        b.iter(|| {
            analyze_content(
                black_box(content.as_bytes()),
                black_box(extension),
                black_box(&thresholds),
                black_box(&path),
                black_box(&config),
                black_box(&disabled_rules),
                black_box(true),
            )
        });
    });
}

criterion_group!(benches, bench_standard_analysis, bench_repetition_heavy);
criterion_main!(benches);
