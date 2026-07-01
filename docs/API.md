<h1 align="center">
    <img width="99" alt="Rust logo" src="https://raw.githubusercontent.com/jamesgober/rust-collection/72baabd71f00e14aa9184efcb16fa3deddda3a0a/assets/rust-logo.svg">
    <br><b>test-lang</b><br>
    <sub><sup>API REFERENCE</sup></sub>
</h1>
<div align="center">
    <sup>
        <a href="../README.md" title="Project Home"><b>HOME</b></a>
        <span>&nbsp;│&nbsp;</span>
        <span>API</span>
        <span>&nbsp;│&nbsp;</span>
        <a href="../CHANGELOG.md" title="Changelog"><b>CHANGELOG</b></a>
    </sup>
</div>
<br>

Compiler snapshot test harness for tokens, ASTs, and diagnostics. This document is the complete reference for the public surface: every exported type, every method, the parameters they take, and at least two runnable examples each.

The design is small on purpose. There are two public types you construct — [`Snapshot`](#snapshot) and, on failure, you receive a [`Mismatch`](#mismatch) — plus the [`Diff`](#diff) / [`Change`](#change) pair that describes what differed. That is the whole API.

<br>

## Table of Contents

- [Installation](#installation)
- [Concepts](#concepts)
  - [Normalization](#normalization)
- [Public APIs](#public-apis)
  - [`Snapshot`](#snapshot)
    - [`Snapshot::new`](#snapshotnew)
    - [`Snapshot::display`](#snapshotdisplay)
    - [`Snapshot::debug`](#snapshotdebug)
    - [`Snapshot::per_line`](#snapshotper_line)
    - [`Snapshot::as_str`](#snapshotas_str)
    - [`Snapshot::check`](#snapshotcheck)
  - [`Mismatch`](#mismatch)
  - [`Diff`](#diff)
  - [`Change`](#change)
- [Recipes](#recipes)

<br>

## Installation

```toml
[dependencies]
test-lang = "0.2"
```

`no_std` + `alloc` (everything works; only the `Error` impl moves from `std` to `core`):

```toml
[dependencies]
test-lang = { version = "0.2", default-features = false }
```

MSRV: Rust 1.85 (2024 edition).

<br>

## Concepts

A **snapshot** is the rendered output of some stage of a compiler front-end, captured as text: a token stream, a pretty-printed syntax tree, a rendered diagnostic. You capture the stage's output into a [`Snapshot`](#snapshot), then [`check`](#snapshotcheck) it against an expected block of text. A match returns `Ok(())`; a mismatch returns a [`Mismatch`](#mismatch) carrying a line-level [`Diff`](#diff).

The crate depends on no other front-end crate. It works over `core::fmt::Display` and `core::fmt::Debug`, so it snapshots whatever a lexer, parser, or diagnostics layer renders — without a compile-time dependency on the crate that produced it.

<a id="normalization"></a>
### Normalization

Both the captured snapshot and the expected text are normalized before comparison, so byte-for-byte equality is not required for a test to pass:

| Rule | Effect |
|---|---|
| Line endings | `\r\n` and lone `\r` become `\n` |
| Trailing whitespace | spaces and tabs at the end of each line are stripped |
| Trailing blank lines | a trailing newline — or several — is removed |

Interior blank lines and **leading** whitespace are preserved: indentation in a pretty-printed tree is significant. This is what makes a snapshot written by hand on Linux match output captured on Windows.

<br>

## Public APIs

<a id="snapshot"></a>
### `Snapshot`

A normalized, comparable rendering of some compiler output.

```rust,ignore
pub struct Snapshot { /* private */ }
```

Derives `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`. Construct one with `new`, `display`, `debug`, or `per_line`; read it back with `as_str`; compare it with `check`. `Snapshot` also implements `Display`, rendering its normalized text.

---

<a id="snapshotnew"></a>
#### `Snapshot::new`

```rust,ignore
pub fn new(text: impl AsRef<str>) -> Snapshot
```

Build a snapshot from an already-rendered block of text. Use this when the stage under test hands you a `String` or `&str` directly — a rendered diagnostic, for example.

**Parameters**

- `text` — any string-like value (`&str`, `String`, `Cow<str>`, …). It is normalized (see [Normalization](#normalization)); the original is not retained.

**Examples**

Trailing whitespace and CRLF endings are normalized away:

```rust
use test_lang::Snapshot;

let snap = Snapshot::new("a  \r\nb\n");
assert_eq!(snap.as_str(), "a\nb");
```

A multi-line diagnostic captured verbatim:

```rust
use test_lang::Snapshot;

let rendered = "error: unexpected token\n  --> 1:5\n  |\n1 | let = 1\n  |     ^ expected identifier";
let snap = Snapshot::new(rendered);
assert!(snap.check(rendered).is_ok());
```

---

<a id="snapshotdisplay"></a>
#### `Snapshot::display`

```rust,ignore
pub fn display(value: &impl core::fmt::Display) -> Snapshot
```

Build a snapshot by rendering a value through its `Display` implementation. The natural entry point for a value that already prints itself the way a test should read it — a single token, a formatted diagnostic, a version.

**Parameters**

- `value` — a reference to any `Display` value.

**Examples**

A value with a custom `Display`:

```rust
use core::fmt;
use test_lang::Snapshot;

struct Version(u32, u32, u32);
impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

let snap = Snapshot::display(&Version(0, 2, 0));
assert_eq!(snap.as_str(), "0.2.0");
```

Any standard `Display` type works:

```rust
use test_lang::Snapshot;

assert_eq!(Snapshot::display(&42).as_str(), "42");
```

---

<a id="snapshotdebug"></a>
#### `Snapshot::debug`

```rust,ignore
pub fn debug(value: &impl core::fmt::Debug) -> Snapshot
```

Build a snapshot by rendering a value through its `Debug` implementation, using the alternate (`{:#?}`) pretty form. Most syntax-tree node types derive `Debug` but not `Display`; this captures the multi-line tree without asking the node to implement a display format of its own.

**Parameters**

- `value` — a reference to any `Debug` value.

**Examples**

A derived-`Debug` tree:

```rust
use test_lang::Snapshot;

#[derive(Debug)]
struct Binary { op: char, lhs: i64, rhs: i64 }

let snap = Snapshot::debug(&Binary { op: '+', lhs: 1, rhs: 2 });
assert!(snap.as_str().contains("op: '+'"));
assert!(snap.as_str().contains("lhs: 1"));
```

Capture, then accept, a tree snapshot (paste `as_str()` back as the expected value):

```rust
use test_lang::Snapshot;

#[derive(Debug)]
enum Expr { Int(i64) }

let snap = Snapshot::debug(&Expr::Int(7));
assert!(snap.check(snap.as_str()).is_ok());
```

---

<a id="snapshotper_line"></a>
#### `Snapshot::per_line`

```rust,ignore
pub fn per_line<I>(items: I) -> Snapshot
where
    I: IntoIterator,
    I::Item: core::fmt::Display,
```

Build a snapshot from a sequence of values, rendering each on its own line through `Display`. This is the idiomatic way to snapshot a **token stream**: one token per line means the diff on failure points at the exact token that changed, instead of at one long line.

**Parameters**

- `items` — anything iterable whose items are `Display` (`Vec<T>`, `&[T]`, an array, an iterator, …).

**Examples**

A token stream rendered one per line:

```rust
use test_lang::Snapshot;

let kinds = ["Ident(x)", "Eq", "Int(1)"];
let snap = Snapshot::per_line(kinds);
assert_eq!(snap.as_str(), "Ident(x)\nEq\nInt(1)");
```

Works over an owned `Vec` of `String`, the shape a real lexer returns:

```rust
use test_lang::Snapshot;

let tokens: Vec<String> = "let x = 1".split_whitespace().map(str::to_string).collect();
let snap = Snapshot::per_line(&tokens);
assert!(snap.check("let\nx\n=\n1").is_ok());
```

An empty sequence yields an empty snapshot:

```rust
use test_lang::Snapshot;

let empty: [&str; 0] = [];
assert_eq!(Snapshot::per_line(empty).as_str(), "");
```

---

<a id="snapshotas_str"></a>
#### `Snapshot::as_str`

```rust,ignore
pub fn as_str(&self) -> &str
```

Return the normalized snapshot text. This is what [`check`](#snapshotcheck) compares, and what you paste into a test as the expected value when accepting a new snapshot.

**Examples**

```rust
use test_lang::Snapshot;

let snap = Snapshot::new("first\nsecond\n\n");
assert_eq!(snap.as_str(), "first\nsecond");
```

Round-trip: a snapshot always matches its own text:

```rust
use test_lang::Snapshot;

let snap = Snapshot::per_line(["a", "b", "c"]);
assert!(snap.check(snap.as_str()).is_ok());
```

---

<a id="snapshotcheck"></a>
#### `Snapshot::check`

```rust,ignore
pub fn check(&self, expected: impl AsRef<str>) -> Result<(), Mismatch>
```

Compare the snapshot against `expected`. The expected text is normalized the same way the snapshot was, so it can be written inline in a test as a plain string literal without worrying about trailing newlines or platform line endings.

**Parameters**

- `expected` — the known-good text, any string-like value.

**Returns**

- `Ok(())` when the normalized snapshot equals the normalized expected text.
- `Err(`[`Mismatch`](#mismatch)`)` otherwise, carrying the line-level diff.

**Errors**

Returns [`Mismatch`](#mismatch) on any difference. Its `Display` renders a unified diff; [`Mismatch::diff`](#mismatch) exposes the [`Diff`](#diff) programmatically.

**Examples**

A matching check, propagated with `?` in a fallible test:

```rust
use test_lang::Snapshot;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    Snapshot::per_line(["a", "b"]).check("a\nb")?;
    Ok(())
}
assert!(run().is_ok());
```

A mismatch carries a diff you can print or inspect:

```rust
use test_lang::Snapshot;

let err = Snapshot::per_line(["a", "b"]).check("a\nc").unwrap_err();
assert!(err.to_string().contains("-c")); // expected `c`, was missing
assert!(err.to_string().contains("+b")); // `b` was produced instead
```

Normalization means trailing-newline and line-ending differences never fail a check:

```rust
use test_lang::Snapshot;

assert!(Snapshot::new("a\nb").check("a\r\nb\n").is_ok());
```

<br>

<a id="mismatch"></a>
### `Mismatch`

The error returned by [`Snapshot::check`](#snapshotcheck) when the snapshot does not match.

```rust,ignore
pub struct Mismatch { /* private */ }

pub fn diff(&self) -> &Diff
```

Derives `Debug`, `Clone`, `PartialEq`, `Eq`. Implements `Display` — printing a unified diff, `-` lines expected and `+` lines produced, under a `snapshot mismatch (-expected +actual):` header — and `core::error::Error`, so it slots into `Result<_, Box<dyn Error>>` and `?` chains.

**Methods**

- `diff(&self) -> &Diff` — the line-level [`Diff`](#diff), for inspecting the mismatch programmatically instead of parsing the rendered string.

**Examples**

Surface the diff straight through a failing test's panic message:

```rust
use test_lang::Snapshot;

let err = Snapshot::new("actual").check("expected").unwrap_err();
assert!(err.to_string().contains("-expected"));
assert!(err.to_string().contains("+actual"));
```

Inspect the mismatch programmatically — count how many lines changed:

```rust
use test_lang::{Change, Snapshot};

let err = Snapshot::new("a\nb").check("a\nB").unwrap_err();
let edits = err.diff().changes().filter(|(c, _)| *c != Change::Equal).count();
assert_eq!(edits, 2); // one deletion, one insertion
```

<br>

<a id="diff"></a>
### `Diff`

A minimal line-level edit script between two blocks of text. Computed by the harness, exposed through [`Mismatch::diff`](#mismatch), and constructable directly for standalone use.

```rust,ignore
pub fn lines(expected: &str, actual: &str) -> Diff
pub fn is_empty(&self) -> bool
pub fn changes(&self) -> impl Iterator<Item = (Change, &str)>
```

Derives `Debug`, `Clone`, `PartialEq`, `Eq`. Implements `Display`, rendering the diff in unified style (one line per change, each prefixed with its [`Change`](#change) marker). The engine is an LCS diff with a common prefix/suffix fast path, so identical leading and trailing lines are matched in linear time and the diff stays tight around the region that changed.

**Methods**

- `lines(expected, actual) -> Diff` — compute the diff. Both inputs are split on `\n`; the caller is expected to have normalized line endings ([`Snapshot`](#snapshot) does this).
- `is_empty(&self) -> bool` — `true` when the two inputs were line-for-line identical (no insertions or deletions).
- `changes(&self) -> impl Iterator<Item = (Change, &str)>` — every line in order, as `(change, text)` pairs. Equal lines are included so the full aligned view can be reconstructed.

**Examples**

Compute and render a diff directly:

```rust
use test_lang::Diff;

let diff = Diff::lines("a\nb\nc", "a\nB\nc");
assert!(!diff.is_empty());

let rendered = diff.to_string();
assert!(rendered.contains(" a"));  // unchanged
assert!(rendered.contains("-b"));  // expected
assert!(rendered.contains("+B"));  // actual
```

Filter to just the inserted lines:

```rust
use test_lang::{Change, Diff};

let diff = Diff::lines("one\ntwo", "one\ntwo\nthree");
let inserted: Vec<_> = diff
    .changes()
    .filter(|(c, _)| *c == Change::Insert)
    .map(|(_, line)| line)
    .collect();
assert_eq!(inserted, ["three"]);
```

Identical inputs produce an empty diff:

```rust
use test_lang::Diff;

assert!(Diff::lines("same\ntext", "same\ntext").is_empty());
```

<br>

<a id="change"></a>
### `Change`

The role a line plays in a [`Diff`](#diff).

```rust,ignore
pub enum Change { Equal, Delete, Insert }

pub const fn marker(self) -> char
```

Derives `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`.

**Variants**

- `Equal` — present, unchanged, in both the expected and actual text.
- `Delete` — present in the expected text but missing from the actual (rendered `-`).
- `Insert` — present in the actual text but not expected (rendered `+`).

**Methods**

- `marker(self) -> char` — the single-character unified-diff marker: `' '` for `Equal`, `'-'` for `Delete`, `'+'` for `Insert`. `const fn`.

**Examples**

```rust
use test_lang::Change;

assert_eq!(Change::Equal.marker(), ' ');
assert_eq!(Change::Delete.marker(), '-');
assert_eq!(Change::Insert.marker(), '+');
```

Match on a change to classify a diff line:

```rust
use test_lang::{Change, Diff};

let diff = Diff::lines("keep\ndrop", "keep\nadd");
for (change, text) in diff.changes() {
    match change {
        Change::Equal => assert_eq!(text, "keep"),
        Change::Delete => assert_eq!(text, "drop"),
        Change::Insert => assert_eq!(text, "add"),
    }
}
```

<br>

## Recipes

**Snapshot a token stream.** Render each token on its own line so the diff pinpoints the changed token.

```rust
use test_lang::Snapshot;

let tokens: Vec<String> = "a + b".split_whitespace().map(str::to_string).collect();
Snapshot::per_line(&tokens).check("a\n+\nb").unwrap();
```

**Snapshot a syntax tree.** Use `debug` for the pretty-printed tree; accept a new tree by pasting `as_str()` into the test.

```rust
use test_lang::Snapshot;

#[derive(Debug)]
enum Expr { Int(i64), Neg(Box<Expr>) }

let tree = Expr::Neg(Box::new(Expr::Int(5)));
let snap = Snapshot::debug(&tree);
assert!(snap.check(snap.as_str()).is_ok());
```

**Snapshot a rendered diagnostic.** Normalization erases platform differences, so the same expected block passes everywhere.

```rust
use test_lang::Snapshot;

// Output captured on Windows: CRLF endings, a stray trailing space.
let captured = "error: bad token  \r\n  --> 1:1\r\n";
Snapshot::new(captured).check("error: bad token\n  --> 1:1").unwrap();
```

<br>

---

<sub>Copyright &copy; 2026 <strong>James Gober</strong>.</sub>
