//! Benchmarks for the snapshot harness hot paths: capturing a snapshot and
//! checking it against expected text.
//!
//! The two workloads that matter in a test suite are (1) building a snapshot
//! from a stage's output and (2) the check itself — including the diff on the
//! matching path, which is the common case a green test suite runs thousands of
//! times. The changed path (a real mismatch) is measured separately because it
//! runs the full LCS engine.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use test_lang::Snapshot;

/// Build a block of `n` numbered lines to stand in for a stage's output.
fn make_lines(n: usize) -> Vec<String> {
    (0..n).map(|i| format!("Token({i})")).collect()
}

fn bench_capture(c: &mut Criterion) {
    let mut group = c.benchmark_group("capture");
    for &n in &[16usize, 256, 4096] {
        let lines = make_lines(n);
        group.bench_with_input(BenchmarkId::from_parameter(n), &lines, |b, lines| {
            b.iter(|| Snapshot::per_line(black_box(lines)));
        });
    }
    group.finish();
}

fn bench_check_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("check_match");
    for &n in &[16usize, 256, 4096] {
        let lines = make_lines(n);
        let snapshot = Snapshot::per_line(&lines);
        let expected = lines.join("\n");
        group.bench_with_input(BenchmarkId::from_parameter(n), &expected, |b, expected| {
            b.iter(|| snapshot.check(black_box(expected)).is_ok());
        });
    }
    group.finish();
}

fn bench_check_mismatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("check_mismatch");
    for &n in &[16usize, 256, 4096] {
        let lines = make_lines(n);
        let snapshot = Snapshot::per_line(&lines);
        // Change a single line in the middle: forces the LCS engine to run but
        // keeps the common prefix/suffix fast path realistic.
        let mut expected_lines = lines.clone();
        expected_lines[n / 2] = String::from("Token(CHANGED)");
        let expected = expected_lines.join("\n");
        group.bench_with_input(BenchmarkId::from_parameter(n), &expected, |b, expected| {
            b.iter(|| snapshot.check(black_box(expected)).is_err());
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_capture,
    bench_check_match,
    bench_check_mismatch
);
criterion_main!(benches);
