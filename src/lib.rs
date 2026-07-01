//! # test_lang
//!
//! A snapshot test harness for language front-ends: give it source, run that
//! source through a stage, and assert the rendered result — the token stream,
//! the syntax tree, or the diagnostics — against an expected block of text.
//!
//! ## Why a snapshot harness
//!
//! The output of a lexer, parser, or diagnostics renderer is a *shape*: a list
//! of tokens, a tree of nodes, a set of caret-annotated errors. Asserting that
//! shape field by field is verbose and brittle — one added node and a dozen
//! index-based assertions shift. A snapshot test instead captures the whole
//! rendered shape as text and compares it against a known-good block. When the
//! output changes, you get a line-level diff pointing at exactly what moved, and
//! accepting the new behavior is a copy-paste.
//!
//! This crate owns no grammar and depends on no other front-end crate. It works
//! on anything that can render itself to text — a
//! [`Display`](core::fmt::Display) value, a [`Debug`](core::fmt::Debug) tree, or
//! an iterator of displayable items — so the same harness serves a hand-written
//! lexer, a generated parser, or a diagnostics layer without coupling to any of
//! them.
//!
//! ## Stability
//!
//! As of 1.0.0 the public surface — the four exported types and their inherent
//! methods and trait impls — is stable and frozen. It will not change in a
//! breaking way before a 2.0. New capability arrives additively in 1.x releases.
//!
//! ## The two types
//!
//! - [`Snapshot`] — a normalized, comparable rendering of some output. Build one
//!   with [`Snapshot::display`], [`Snapshot::debug`], [`Snapshot::per_line`], or
//!   [`Snapshot::new`], then call [`Snapshot::check`].
//! - [`Diff`] — the line-level difference reported when a check fails, rendered
//!   as a unified `-expected`/`+actual` diff. A failed check hands it back inside
//!   a [`Mismatch`].
//!
//! ## Example
//!
//! Snapshot a token stream and assert it:
//!
//! ```
//! use test_lang::Snapshot;
//!
//! // Whatever your lexer produces — here, a stand-in that yields display strings.
//! fn lex(source: &str) -> Vec<String> {
//!     source.split_whitespace().map(str::to_string).collect()
//! }
//!
//! let tokens = lex("let x = 1");
//! let snapshot = Snapshot::per_line(&tokens);
//!
//! snapshot.check("let\nx\n=\n1").expect("token stream matches");
//! ```
//!
//! When the output drifts, the returned [`Mismatch`] shows precisely what
//! changed:
//!
//! ```
//! use test_lang::Snapshot;
//!
//! let snapshot = Snapshot::per_line(["let", "y", "=", "1"]);
//! let err = snapshot.check("let\nx\n=\n1").unwrap_err();
//!
//! // `-x` was expected; `+y` was produced in its place.
//! assert!(err.to_string().contains("-x"));
//! assert!(err.to_string().contains("+y"));
//! ```
//!
//! ## `no_std`
//!
//! The crate is `no_std` + `alloc` when the default `std` feature is disabled.
//! Every type — including the [`Error`](core::error::Error) impl on
//! [`Mismatch`], which is anchored to `core::error::Error` in both modes — is
//! available with or without `std`.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]

extern crate alloc;

mod diff;
mod snapshot;

pub use diff::{Change, Diff};
pub use snapshot::{Mismatch, Snapshot};
