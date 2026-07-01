//! Line-level difference between two blocks of text.
//!
//! A snapshot assertion is only as useful as the report it prints when it
//! fails. Comparing two multi-line strings and saying "they differ" is not
//! enough — the reader needs to see *which* lines were expected, which were
//! produced, and where the two streams line up again. That is what [`Diff`]
//! provides: a minimal edit script over lines, rendered in the unified `-`/`+`
//! style every developer already knows from `git diff`.
//!
//! The engine is a longest-common-subsequence (LCS) diff with a common
//! prefix/suffix fast path. Identical leading and trailing lines are matched in
//! linear time before the quadratic LCS ever runs, so the expensive step only
//! sees the region that actually changed. For snapshot-sized inputs this is
//! effectively instant, and it keeps the reported diff tight instead of
//! re-aligning lines that never moved.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

/// The role a line plays in a [`Diff`].
///
/// A diff is a sequence of lines, each tagged with how it relates to the two
/// inputs. "Expected" is the baseline (the left/old side); "actual" is the
/// value under test (the right/new side).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Change {
    /// The line is present, unchanged, in both the expected and actual text.
    Equal,
    /// The line is present in the expected text but missing from the actual
    /// text. Rendered with a leading `-`.
    Delete,
    /// The line is present in the actual text but was not expected. Rendered
    /// with a leading `+`.
    Insert,
}

impl Change {
    /// The single-character marker used when rendering this change in a unified
    /// diff: `' '` for [`Equal`](Change::Equal), `'-'` for
    /// [`Delete`](Change::Delete), `'+'` for [`Insert`](Change::Insert).
    #[inline]
    #[must_use]
    pub const fn marker(self) -> char {
        match self {
            Change::Equal => ' ',
            Change::Delete => '-',
            Change::Insert => '+',
        }
    }
}

/// A minimal line-level edit script between two blocks of text.
///
/// Construct one with [`Diff::lines`]. Iterate the individual changes with
/// [`changes`](Diff::changes), ask whether the two inputs matched with
/// [`is_empty`](Diff::is_empty), or format the whole thing as a unified diff
/// through its [`Display`](fmt::Display) implementation.
///
/// # Examples
///
/// ```
/// use test_lang::{Change, Diff};
///
/// let diff = Diff::lines("a\nb\nc", "a\nB\nc");
/// assert!(!diff.is_empty());
///
/// // The unchanged `a` and `c` frame the one changed line.
/// let rendered = diff.to_string();
/// assert!(rendered.contains("-b"));
/// assert!(rendered.contains("+B"));
/// assert!(rendered.contains(" a"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct Diff {
    changes: Vec<(Change, String)>,
}

impl Diff {
    /// Compute the line-level diff between `expected` and `actual`.
    ///
    /// Both inputs are split on `\n` into lines (the caller is expected to have
    /// already normalized line endings — [`Snapshot`](crate::Snapshot) does
    /// this for you). Common leading and trailing lines are matched directly;
    /// only the differing middle region is run through the LCS engine.
    ///
    /// # Examples
    ///
    /// Two identical strings produce an empty diff:
    ///
    /// ```
    /// use test_lang::Diff;
    ///
    /// let diff = Diff::lines("same\ntext", "same\ntext");
    /// assert!(diff.is_empty());
    /// ```
    ///
    /// An added line shows up as an insertion:
    ///
    /// ```
    /// use test_lang::{Change, Diff};
    ///
    /// let diff = Diff::lines("one\ntwo", "one\ntwo\nthree");
    /// let inserted: Vec<_> = diff
    ///     .changes()
    ///     .filter(|(c, _)| *c == Change::Insert)
    ///     .map(|(_, line)| line)
    ///     .collect();
    /// assert_eq!(inserted, ["three"]);
    /// ```
    pub fn lines(expected: &str, actual: &str) -> Self {
        let old: Vec<&str> = split_lines(expected);
        let new: Vec<&str> = split_lines(actual);

        let mut changes = Vec::new();

        // Fast path: peel off the identical prefix without touching the LCS
        // table. This is the common case for a near-miss snapshot.
        let mut start = 0;
        let max_prefix = old.len().min(new.len());
        while start < max_prefix && old[start] == new[start] {
            start += 1;
        }

        // Peel off the identical suffix, stopping before the shared prefix so a
        // line is never counted on both ends.
        let mut old_end = old.len();
        let mut new_end = new.len();
        while old_end > start && new_end > start && old[old_end - 1] == new[new_end - 1] {
            old_end -= 1;
            new_end -= 1;
        }

        for line in &old[..start] {
            changes.push((Change::Equal, String::from(*line)));
        }

        diff_middle(&old[start..old_end], &new[start..new_end], &mut changes);

        for line in &old[old_end..] {
            changes.push((Change::Equal, String::from(*line)));
        }

        Diff { changes }
    }

    /// Returns `true` when the two inputs were line-for-line identical, i.e. the
    /// diff contains no insertions or deletions.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_lang::Diff;
    ///
    /// assert!(Diff::lines("x", "x").is_empty());
    /// assert!(!Diff::lines("x", "y").is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.changes.iter().all(|(c, _)| *c == Change::Equal)
    }

    /// Iterate over every line in the diff, in order, as `(change, text)` pairs.
    ///
    /// Equal lines are included so the caller can reconstruct the full aligned
    /// view; filter on the [`Change`] if only edits are of interest.
    ///
    /// # Examples
    ///
    /// ```
    /// use test_lang::{Change, Diff};
    ///
    /// let diff = Diff::lines("keep\ndrop", "keep");
    /// let deleted = diff
    ///     .changes()
    ///     .filter(|(c, _)| *c == Change::Delete)
    ///     .count();
    /// assert_eq!(deleted, 1);
    /// ```
    pub fn changes(&self) -> impl Iterator<Item = (Change, &str)> {
        self.changes.iter().map(|(c, s)| (*c, s.as_str()))
    }
}

impl fmt::Display for Diff {
    /// Render the diff in unified style: one line per change, each prefixed with
    /// its [`marker`](Change::marker) and a space.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, (change, text)) in self.changes.iter().enumerate() {
            if i > 0 {
                f.write_str("\n")?;
            }
            f.write_fmt(format_args!("{}{}", change.marker(), text))?;
        }
        Ok(())
    }
}

/// Split a block of text into lines without allocating.
///
/// An empty string is treated as zero lines (not one empty line) so that an
/// empty snapshot compares clean against another empty snapshot.
fn split_lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        Vec::new()
    } else {
        text.split('\n').collect()
    }
}

/// Diff the non-trivial middle region with an LCS edit script.
///
/// `old` and `new` have already had their common prefix and suffix removed, so
/// the first and last lines here are guaranteed to differ (unless one side is
/// empty). Deletions are emitted before insertions at each divergence point,
/// which reads naturally in the unified output.
fn diff_middle<'a>(old: &[&'a str], new: &[&'a str], out: &mut Vec<(Change, String)>) {
    if old.is_empty() {
        out.extend(new.iter().map(|l| (Change::Insert, String::from(*l))));
        return;
    }
    if new.is_empty() {
        out.extend(old.iter().map(|l| (Change::Delete, String::from(*l))));
        return;
    }

    let table = lcs_table(old, new);

    // Walk the LCS table from the top-left, emitting an edit script. At each
    // cell we either take a matching line (diagonal) or step in the direction
    // the table says preserves the most shared lines.
    let mut i = 0;
    let mut j = 0;
    while i < old.len() && j < new.len() {
        if old[i] == new[j] {
            out.push((Change::Equal, String::from(old[i])));
            i += 1;
            j += 1;
        } else if table[i + 1][j] >= table[i][j + 1] {
            out.push((Change::Delete, String::from(old[i])));
            i += 1;
        } else {
            out.push((Change::Insert, String::from(new[j])));
            j += 1;
        }
    }
    out.extend(old[i..].iter().map(|l| (Change::Delete, String::from(*l))));
    out.extend(new[j..].iter().map(|l| (Change::Insert, String::from(*l))));
}

/// Build the LCS-length dynamic-programming table for two line slices.
///
/// `table[i][j]` holds the length of the longest common subsequence of
/// `old[i..]` and `new[j..]`, so the forward walk in [`diff_middle`] can pick
/// the direction that retains the most shared lines. The table is
/// `(old.len() + 1) x (new.len() + 1)`; the extra row and column are the empty
/// base cases.
fn lcs_table(old: &[&str], new: &[&str]) -> Vec<Vec<u32>> {
    let rows = old.len() + 1;
    let cols = new.len() + 1;
    let mut table = alloc::vec![alloc::vec![0u32; cols]; rows];

    for i in (0..old.len()).rev() {
        for j in (0..new.len()).rev() {
            table[i][j] = if old[i] == new[j] {
                table[i + 1][j + 1] + 1
            } else {
                table[i + 1][j].max(table[i][j + 1])
            };
        }
    }
    table
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    #[test]
    fn test_diff_identical_is_empty() {
        assert!(Diff::lines("a\nb\nc", "a\nb\nc").is_empty());
    }

    #[test]
    fn test_diff_empty_inputs_is_empty() {
        assert!(Diff::lines("", "").is_empty());
    }

    #[test]
    fn test_diff_single_line_change_reports_both_sides() {
        let diff = Diff::lines("a\nb\nc", "a\nB\nc");
        let changes: Vec<_> = diff.changes().collect();
        assert_eq!(
            changes,
            vec![
                (Change::Equal, "a"),
                (Change::Delete, "b"),
                (Change::Insert, "B"),
                (Change::Equal, "c"),
            ]
        );
    }

    #[test]
    fn test_diff_insertion_at_end() {
        let diff = Diff::lines("a\nb", "a\nb\nc");
        let inserted: Vec<_> = diff
            .changes()
            .filter(|(c, _)| *c == Change::Insert)
            .map(|(_, l)| l)
            .collect();
        assert_eq!(inserted, vec!["c"]);
    }

    #[test]
    fn test_diff_deletion_at_start() {
        let diff = Diff::lines("a\nb\nc", "b\nc");
        let deleted: Vec<_> = diff
            .changes()
            .filter(|(c, _)| *c == Change::Delete)
            .map(|(_, l)| l)
            .collect();
        assert_eq!(deleted, vec!["a"]);
    }

    #[test]
    fn test_diff_all_different() {
        let diff = Diff::lines("a\nb", "x\ny");
        assert!(!diff.is_empty());
        assert_eq!(
            diff.changes().filter(|(c, _)| *c == Change::Delete).count(),
            2
        );
        assert_eq!(
            diff.changes().filter(|(c, _)| *c == Change::Insert).count(),
            2
        );
    }

    #[test]
    fn test_diff_expected_empty_all_insert() {
        let diff = Diff::lines("", "a\nb");
        let changes: Vec<_> = diff.changes().collect();
        assert_eq!(changes, vec![(Change::Insert, "a"), (Change::Insert, "b")]);
    }

    #[test]
    fn test_diff_actual_empty_all_delete() {
        let diff = Diff::lines("a\nb", "");
        let changes: Vec<_> = diff.changes().collect();
        assert_eq!(changes, vec![(Change::Delete, "a"), (Change::Delete, "b")]);
    }

    #[test]
    fn test_diff_display_uses_markers() {
        let diff = Diff::lines("a\nb", "a\nc");
        let rendered = diff.to_string();
        assert_eq!(rendered, " a\n-b\n+c");
    }

    #[test]
    fn test_change_marker() {
        assert_eq!(Change::Equal.marker(), ' ');
        assert_eq!(Change::Delete.marker(), '-');
        assert_eq!(Change::Insert.marker(), '+');
    }

    #[test]
    fn test_diff_preserves_common_prefix_and_suffix() {
        let diff = Diff::lines("head\nold\ntail", "head\nnew\ntail");
        let changes: Vec<_> = diff.changes().collect();
        assert_eq!(
            changes,
            vec![
                (Change::Equal, "head"),
                (Change::Delete, "old"),
                (Change::Insert, "new"),
                (Change::Equal, "tail"),
            ]
        );
    }

    #[test]
    fn test_diff_lcs_finds_shared_middle() {
        // The shared "keep" line should be matched, not deleted-and-reinserted.
        let diff = Diff::lines("a\nkeep\nb", "x\nkeep\ny");
        assert_eq!(
            diff.changes().filter(|(c, _)| *c == Change::Equal).count(),
            1
        );
    }
}
