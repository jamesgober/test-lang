//! Capturing and asserting textual snapshots of compiler output.
//!
//! A [`Snapshot`] is a normalized block of text — the rendered form of some
//! stage's output, such as a token stream, a syntax tree, or a set of
//! diagnostics. The point of normalizing is that a test can be written once and
//! pass on every platform: a snapshot captured on Windows (CRLF line endings,
//! stray trailing whitespace from a formatter) compares equal to the same
//! snapshot written by hand in a test on Linux.
//!
//! The typical flow is: run the source through the stage under test, wrap the
//! result in a `Snapshot`, and call [`Snapshot::check`] against the expected
//! text. On a match you get `Ok(())`; on a mismatch you get a [`Mismatch`]
//! whose [`Display`](fmt::Display) is a ready-to-read unified diff.

use crate::diff::Diff;
use alloc::string::String;
use core::fmt;

/// A normalized, comparable rendering of some compiler output.
///
/// Snapshots normalize three things so that byte-for-byte equality is not
/// required for a test to pass:
///
/// - **Line endings.** `\r\n` and lone `\r` both become `\n`.
/// - **Trailing whitespace.** Spaces and tabs at the end of each line are
///   stripped, so an editor that trims (or fails to trim) lines does not break
///   a test.
/// - **Trailing blank lines.** A trailing newline — or several — is removed, so
///   the expected text can be written with or without one.
///
/// Interior blank lines and leading whitespace are preserved: they are usually
/// significant (indentation in a pretty-printed tree, for example).
///
/// # Examples
///
/// Capture a value that implements [`Display`](fmt::Display):
///
/// ```
/// use test_lang::Snapshot;
///
/// let snap = Snapshot::display(&42);
/// assert_eq!(snap.as_str(), "42");
/// ```
///
/// Capture a token stream, one item per line:
///
/// ```
/// use test_lang::Snapshot;
///
/// let tokens = ["let", "x", "=", "1"];
/// let snap = Snapshot::per_line(tokens);
/// assert_eq!(snap.as_str(), "let\nx\n=\n1");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Snapshot {
    text: String,
}

impl Snapshot {
    /// Build a snapshot from an already-rendered block of text.
    ///
    /// The input is normalized (see the type-level docs). Use this when the
    /// stage under test already hands you a `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_lang::Snapshot;
    ///
    /// // Trailing whitespace and CRLF endings are normalized away.
    /// let snap = Snapshot::new("a  \r\nb\n");
    /// assert_eq!(snap.as_str(), "a\nb");
    /// ```
    pub fn new(text: impl AsRef<str>) -> Self {
        Snapshot {
            text: normalize(text.as_ref()),
        }
    }

    /// Build a snapshot by rendering a value through its
    /// [`Display`](fmt::Display) implementation.
    ///
    /// This is the natural entry point for a value that already knows how to
    /// print itself the way a test should read it — a formatted diagnostic, a
    /// single token, a pretty-printed tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use core::fmt;
    /// use test_lang::Snapshot;
    ///
    /// struct Diagnostic;
    /// impl fmt::Display for Diagnostic {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "error: unexpected token")
    ///     }
    /// }
    ///
    /// let snap = Snapshot::display(&Diagnostic);
    /// assert_eq!(snap.as_str(), "error: unexpected token");
    /// ```
    pub fn display(value: &impl fmt::Display) -> Self {
        Snapshot::new(alloc::format!("{value}"))
    }

    /// Build a snapshot by rendering a value through its
    /// [`Debug`](fmt::Debug) implementation, using the alternate (`{:#?}`)
    /// pretty form.
    ///
    /// Most syntax-tree node types derive `Debug` but not `Display`; this
    /// captures the multi-line pretty-printed tree without requiring the node to
    /// implement a display format of its own.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_lang::Snapshot;
    ///
    /// #[derive(Debug)]
    /// struct Binary { op: char }
    ///
    /// let snap = Snapshot::debug(&Binary { op: '+' });
    /// assert!(snap.as_str().contains("op: '+'"));
    /// ```
    pub fn debug(value: &impl fmt::Debug) -> Self {
        Snapshot::new(alloc::format!("{value:#?}"))
    }

    /// Build a snapshot from a sequence of values, rendering each on its own
    /// line through [`Display`](fmt::Display).
    ///
    /// This is the idiomatic way to snapshot a token stream: one token per
    /// line makes the diff on failure point at the exact token that changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_lang::Snapshot;
    ///
    /// let kinds = ["Ident(x)", "Eq", "Int(1)"];
    /// let snap = Snapshot::per_line(kinds);
    /// assert_eq!(snap.as_str(), "Ident(x)\nEq\nInt(1)");
    /// ```
    ///
    /// An empty sequence yields an empty snapshot:
    ///
    /// ```
    /// use test_lang::Snapshot;
    ///
    /// let empty: [&str; 0] = [];
    /// assert_eq!(Snapshot::per_line(empty).as_str(), "");
    /// ```
    pub fn per_line<I>(items: I) -> Self
    where
        I: IntoIterator,
        I::Item: fmt::Display,
    {
        let mut text = String::new();
        for (i, item) in items.into_iter().enumerate() {
            if i > 0 {
                text.push('\n');
            }
            // `write!` into a `String` never fails; the result is discarded on
            // the infallible path deliberately.
            let _ = fmt::write(&mut text, format_args!("{item}"));
        }
        Snapshot::new(text)
    }

    /// The normalized snapshot text.
    ///
    /// This is what [`check`](Snapshot::check) compares, and what you paste back
    /// into a test as the expected value when accepting a new snapshot.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Compare the snapshot against the `expected` text.
    ///
    /// The expected text is normalized the same way the snapshot was, so it can
    /// be written inline in a test as a plain string literal without worrying
    /// about trailing newlines or platform line endings. Returns `Ok(())` on a
    /// match, or a [`Mismatch`] carrying the diff otherwise.
    ///
    /// # Errors
    ///
    /// Returns [`Mismatch`] when the normalized snapshot differs from the
    /// normalized `expected` text. The error's [`Display`](fmt::Display) renders
    /// a unified diff; [`Mismatch::diff`] exposes it programmatically.
    ///
    /// # Examples
    ///
    /// A matching snapshot:
    ///
    /// ```
    /// use test_lang::Snapshot;
    ///
    /// let snap = Snapshot::per_line(["a", "b"]);
    /// assert!(snap.check("a\nb").is_ok());
    /// ```
    ///
    /// A mismatch carries a diff you can print or inspect:
    ///
    /// ```
    /// use test_lang::Snapshot;
    ///
    /// let snap = Snapshot::per_line(["a", "b"]);
    /// let err = snap.check("a\nc").unwrap_err();
    /// assert!(err.to_string().contains("-c"));  // expected `c`, was missing
    /// assert!(err.to_string().contains("+b"));  // `b` was produced instead
    /// ```
    pub fn check(&self, expected: impl AsRef<str>) -> Result<(), Mismatch> {
        let expected = normalize(expected.as_ref());
        let diff = Diff::lines(&expected, &self.text);
        if diff.is_empty() {
            Ok(())
        } else {
            Err(Mismatch { diff })
        }
    }
}

impl fmt::Display for Snapshot {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// The error returned by [`Snapshot::check`] when the snapshot does not match
/// the expected text.
///
/// A `Mismatch` owns the computed [`Diff`]. Its [`Display`](fmt::Display) prints
/// a unified diff — `-` lines were expected, `+` lines were produced — suitable
/// for surfacing straight through a failing test's panic message.
///
/// # Examples
///
/// ```
/// use test_lang::Snapshot;
///
/// let err = Snapshot::new("actual").check("expected").unwrap_err();
/// // `-expected` was wanted; `+actual` is what the stage produced.
/// assert!(err.to_string().contains("-expected"));
/// assert!(err.to_string().contains("+actual"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct Mismatch {
    diff: Diff,
}

impl Mismatch {
    /// The line-level diff between the expected text and the snapshot.
    ///
    /// Use this to inspect the mismatch programmatically instead of parsing the
    /// rendered string — for example, to count how many lines changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_lang::{Change, Snapshot};
    ///
    /// let err = Snapshot::new("a\nb").check("a\nB").unwrap_err();
    /// let edits = err
    ///     .diff()
    ///     .changes()
    ///     .filter(|(c, _)| *c != Change::Equal)
    ///     .count();
    /// assert_eq!(edits, 2); // one deletion, one insertion
    /// ```
    #[inline]
    pub fn diff(&self) -> &Diff {
        &self.diff
    }
}

impl fmt::Display for Mismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("snapshot mismatch (-expected +actual):\n")?;
        fmt::Display::fmt(&self.diff, f)
    }
}

impl core::error::Error for Mismatch {}

/// Normalize a block of text for comparison.
///
/// Converts CRLF and lone CR to LF, strips trailing horizontal whitespace from
/// every line, and drops any trailing blank lines. This is what makes a
/// snapshot stable across platforms and editors.
fn normalize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());

    // Normalize line endings while splitting: treat `\r\n` and `\r` as `\n`.
    let unified = text.replace("\r\n", "\n");
    let unified = unified.replace('\r', "\n");

    for line in unified.split('\n') {
        out.push_str(line.trim_end_matches([' ', '\t']));
        out.push('\n');
    }

    // Drop the trailing blank lines produced above (and any the caller wrote).
    let trimmed = out.trim_end_matches('\n');
    out.truncate(trimmed.len());
    out
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_new_normalizes_crlf_to_lf() {
        assert_eq!(Snapshot::new("a\r\nb").as_str(), "a\nb");
    }

    #[test]
    fn test_new_normalizes_lone_cr() {
        assert_eq!(Snapshot::new("a\rb").as_str(), "a\nb");
    }

    #[test]
    fn test_new_strips_trailing_whitespace_per_line() {
        assert_eq!(Snapshot::new("a \t\nb  ").as_str(), "a\nb");
    }

    #[test]
    fn test_new_strips_trailing_newlines() {
        assert_eq!(Snapshot::new("a\nb\n\n\n").as_str(), "a\nb");
    }

    #[test]
    fn test_new_preserves_interior_blank_lines() {
        assert_eq!(Snapshot::new("a\n\nb").as_str(), "a\n\nb");
    }

    #[test]
    fn test_new_preserves_leading_indentation() {
        assert_eq!(Snapshot::new("    indented").as_str(), "    indented");
    }

    #[test]
    fn test_new_empty_input_is_empty() {
        assert_eq!(Snapshot::new("").as_str(), "");
        assert_eq!(Snapshot::new("\n\n").as_str(), "");
    }

    #[test]
    fn test_display_renders_value() {
        assert_eq!(Snapshot::display(&123).as_str(), "123");
    }

    #[test]
    fn test_debug_uses_pretty_form() {
        // The field is read only through the derived `Debug`, which the
        // dead-code lint does not count; the test asserts on that rendering.
        #[derive(Debug)]
        #[allow(dead_code)]
        struct Node {
            kind: u8,
        }
        let snap = Snapshot::debug(&Node { kind: 7 });
        assert!(snap.as_str().contains("kind: 7"));
    }

    #[test]
    fn test_per_line_joins_with_newlines() {
        assert_eq!(Snapshot::per_line(["a", "b", "c"]).as_str(), "a\nb\nc");
    }

    #[test]
    fn test_per_line_empty_is_empty() {
        let empty: [&str; 0] = [];
        assert_eq!(Snapshot::per_line(empty).as_str(), "");
    }

    #[test]
    fn test_check_matching_returns_ok() {
        assert!(Snapshot::new("a\nb").check("a\nb").is_ok());
    }

    #[test]
    fn test_check_matching_ignores_trailing_newline_difference() {
        assert!(Snapshot::new("a\nb").check("a\nb\n").is_ok());
    }

    #[test]
    fn test_check_matching_ignores_line_ending_difference() {
        assert!(Snapshot::new("a\nb").check("a\r\nb").is_ok());
    }

    #[test]
    fn test_check_mismatch_returns_err() {
        let err = Snapshot::new("a\nb").check("a\nc").unwrap_err();
        assert!(err.to_string().contains("-c"));
        assert!(err.to_string().contains("+b"));
    }

    #[test]
    fn test_check_mismatch_header_present() {
        let err = Snapshot::new("x").check("y").unwrap_err();
        assert!(err.to_string().starts_with("snapshot mismatch"));
    }

    #[test]
    fn test_mismatch_diff_accessor() {
        let err = Snapshot::new("a").check("b").unwrap_err();
        assert!(!err.diff().is_empty());
    }

    #[test]
    fn test_display_impl_matches_as_str() {
        let snap = Snapshot::new("a\nb");
        assert_eq!(snap.to_string(), snap.as_str());
    }
}
