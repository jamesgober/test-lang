<h1 align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br>
    <b>test-lang</b>
    <br>
    <sub><sup>SNAPSHOT HARNESS</sup></sub>
</h1>

<div align="center">
    <a href="https://crates.io/crates/test-lang"><img alt="Crates.io" src="https://img.shields.io/crates/v/test-lang"></a>
    <a href="https://crates.io/crates/test-lang"><img alt="Downloads" src="https://img.shields.io/crates/d/test-lang?color=%230099ff"></a>
    <a href="https://docs.rs/test-lang"><img alt="docs.rs" src="https://img.shields.io/docsrs/test-lang"></a>
    <a href="https://github.com/jamesgober/test-lang/actions"><img alt="CI" src="https://github.com/jamesgober/test-lang/actions/workflows/ci.yml/badge.svg"></a>
    <a href="https://github.com/rust-lang/rfcs/blob/master/text/2495-min-rust-version.md"><img alt="MSRV" src="https://img.shields.io/badge/MSRV-1.85%2B-blue"></a>
</div>

<br>

<div align="left">
    <p>
        <strong>test-lang</strong> is a snapshot test harness for language front-ends. Give it source, run that source through a stage — a lexer, a parser, a diagnostics renderer — and assert the rendered result against a known-good block of text. When the output changes, you get a line-level diff pointing at exactly what moved, and accepting the new behavior is a copy-paste.
    </p>
    <p>
        It owns no grammar and takes no runtime dependencies. The harness works over anything that renders itself to text — a <code>Display</code> value, a <code>Debug</code> tree, or an iterator of displayable items — so the same two types serve a hand-written lexer, a generated parser, or a diagnostics layer without coupling to any of them. It is <code>no_std</code> + <code>alloc</code> capable and normalizes line endings and trailing whitespace so a snapshot captured on Windows matches one written on Linux.
    </p>
    <br>
    <hr>
    <p>
        <strong>MSRV is 1.85+</strong> (Rust 2024 edition). Part of the <code>-lang</code> language-construction family.
    </p>
    <blockquote>
        <strong>Status: pre-1.0, in active development.</strong> The public API is being designed across the 0.x series and frozen at <code>1.0.0</code>. See <a href="./CHANGELOG.md"><code>CHANGELOG.md</code></a>.
    </blockquote>
</div>

<hr>
<br>

## Performance First

Latest local Criterion means (`cargo bench`, Windows x86_64, Rust stable). The workload that matters in a test suite is the check on the matching path — the case a green suite runs on every pass:

- **Capture** (`per_line`, 16 lines): ~0.5 µs
- **Check, matching** (16 lines): ~1.2 µs
- **Check, matching** (256 lines): ~19 µs

The check is a line-level diff with a common prefix/suffix fast path: identical leading and trailing lines are matched in linear time before the quadratic LCS engine runs, so a near-miss snapshot only pays for the region that actually changed.

<br>
<hr>
<br>

## Installation

```toml
[dependencies]
test-lang = "0.2"
```

`no_std` + `alloc` (drops the `std::error::Error` anchor, keeps everything else):

```toml
[dependencies]
test-lang = { version = "0.2", default-features = false }
```

<br>

## Quick Start

Snapshot a token stream, one token per line, and assert it:

```rust
use test_lang::Snapshot;

// Whatever your lexer produces — here a stand-in that yields display strings.
fn lex(source: &str) -> Vec<String> {
    source.split_whitespace().map(str::to_string).collect()
}

let snapshot = Snapshot::per_line(lex("let x = 1"));
snapshot.check("let\nx\n=\n1").expect("token stream matches");
```

When the output drifts, the returned `Mismatch` shows precisely what changed:

```rust
use test_lang::Snapshot;

let snapshot = Snapshot::per_line(["let", "y", "=", "1"]);
let err = snapshot.check("let\nx\n=\n1").unwrap_err();

// `-x` was expected; `+y` was produced in its place.
assert!(err.to_string().contains("-x"));
assert!(err.to_string().contains("+y"));
```

<br>
<hr>

## Features

- **Three capture modes** — `Snapshot::display` for a `Display` value, `Snapshot::debug` for a pretty-printed `Debug` tree, and `Snapshot::per_line` for a token stream (one item per line, so the diff points at the exact token).
- **Cross-platform normalization** — CRLF/CR collapse to LF, trailing whitespace is stripped, and trailing blank lines are trimmed. A snapshot written by hand in a test matches output captured on any platform.
- **Line-level diffs** — a failed `check` returns a `Mismatch` whose `Display` is a unified `-expected`/`+actual` diff; `Mismatch::diff` exposes the `Diff` for programmatic inspection.
- **No runtime dependencies** — built on `core::fmt` and `alloc` only. `no_std` capable.
- **No panics** — `check` returns `Result`; the test author decides whether to `unwrap`, `expect`, or propagate.

<br>

## API Overview

For a complete reference with examples, see [`docs/API.md`](./docs/API.md).

- [`Snapshot`](./docs/API.md#snapshot) — a normalized, comparable rendering of compiler output; `new` / `display` / `debug` / `per_line` / `check`.
- [`Diff`](./docs/API.md#diff) & [`Change`](./docs/API.md#change) — the line-level edit script, rendered as a unified diff.
- [`Mismatch`](./docs/API.md#mismatch) — the error returned by `Snapshot::check`, carrying the `Diff`.

Runnable examples: [`examples/tokens.rs`](./examples/tokens.rs), [`examples/ast.rs`](./examples/ast.rs), [`examples/diagnostics.rs`](./examples/diagnostics.rs).

```bash
cargo run --example tokens
cargo run --example ast
cargo run --example diagnostics
```

<hr>
<br>

## Contributing

See [`REPS.md`](./REPS.md) for engineering standards and the definition of done. Before a PR: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-features` must be clean.

<br>

<div id="license">
    <h2>License</h2>
    <p>Licensed under either of</p>
    <ul>
        <li><b>Apache License, Version 2.0</b> &mdash; <a href="./LICENSE-APACHE">LICENSE-APACHE</a></li>
        <li><b>MIT License</b> &mdash; <a href="./LICENSE-MIT">LICENSE-MIT</a></li>
    </ul>
    <p>at your option.</p>
</div>

<div align="center">
  <h2></h2>
  <sup>COPYRIGHT <small>&copy;</small> 2026 <strong>James Gober <me@jamesgober.com>.</strong></sup>
</div>
