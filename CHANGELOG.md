<h1 align="center">
    <img width="90px" height="auto" src="https://raw.githubusercontent.com/jamesgober/jamesgober/main/media/icons/hexagon-3.svg" alt="Triple Hexagon">
    <br><b>CHANGELOG</b>
</h1>
<p>
  All notable changes to <code>test-lang</code> will be documented in this file. The format is based on <a href="https://keepachangelog.com/en/1.1.0/">Keep a Changelog</a>,
  and this project adheres to <a href="https://semver.org/spec/v2.0.0.html/">Semantic Versioning</a>.
</p>

---

## [Unreleased]

### Added

### Changed

### Fixed

### Security

---

## [1.0.0] - 2026-07-01

API freeze. The public surface shipped in 0.2.0 is now stable: the four types
(`Snapshot`, `Diff`, `Change`, `Mismatch`), their inherent methods, and their
trait implementations will not change in a breaking way before a 2.0. There are
no functional changes from 0.2.0 — this release records the SemVer promise.

### Changed

- `docs/API.md` marked stable; the SemVer stability promise is recorded there and
  in the crate-level documentation.
- Corrected the `std` feature description: the `Error` impl is
  `core::error::Error` in both `std` and `no_std` builds; the feature only
  controls whether the standard library is linked.

---

## [0.2.0] - 2026-07-01

The core snapshot harness. Given a stage's output — a token stream, a syntax
tree, or a rendered diagnostic — the harness captures it as normalized text and
asserts it against an expected block, reporting a line-level unified diff when
they differ. No runtime dependencies; `no_std` + `alloc` supported.

### Added

- `Snapshot` — a normalized, comparable rendering of compiler output. Built with
  `new`, `display`, `debug`, or `per_line`; compared with `check`.
- `Diff` and `Change` — the line-level edit script (LCS with a common
  prefix/suffix fast path), rendered as a unified `-expected`/`+actual` diff.
- `Mismatch` — the error returned by `Snapshot::check`, carrying the `Diff` and
  implementing `core::error::Error`.
- Cross-platform normalization: CRLF/CR to LF, trailing-whitespace stripping,
  and trailing-blank-line trimming, so a snapshot written on one platform passes
  on another.
- `tests/harness.rs` integration tests, `tests/proptests.rs` property tests, and
  `benches/bench.rs` criterion benchmarks for the capture and check paths.
- `examples/tokens.rs`, `examples/ast.rs`, and `examples/diagnostics.rs`.

### Changed

- Removed the unused `serde` feature and optional dependency; the harness has no
  runtime dependencies.
- MSRV alignment: `clippy.toml` now matches the declared `rust-version = 1.85`.

### Fixed

- `Cargo.toml` `keywords` and `categories` arrays now use quoted strings (the
  scaffold left them unquoted, which failed to parse).

---

## [0.1.0] - 2026-06-18

Initial scaffold and repository bootstrap. No domain logic yet &mdash; this release establishes the structure, tooling, and quality gates the implementation will be built on.

### Added

- `Cargo.toml` with crate metadata, Rust 2024 edition, MSRV 1.85.
- Dual `Apache-2.0 OR MIT` license files.
- `README.md`, `CHANGELOG.md`, and a documentation skeleton.
- `REPS.md` compliance baseline.
- `.github/workflows/ci.yml` CI matrix; `deny.toml`, `clippy.toml`, `rustfmt.toml`.
- `dev/DIRECTIVES.md` and `dev/ROADMAP.md` (committed engineering standards + plan).

[Unreleased]: https://github.com/jamesgober/test-lang/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/jamesgober/test-lang/compare/v0.2.0...v1.0.0
[0.2.0]: https://github.com/jamesgober/test-lang/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/jamesgober/test-lang/releases/tag/v0.1.0
