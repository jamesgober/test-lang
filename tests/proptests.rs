//! Property-based tests for the harness invariants.
//!
//! These exercise the two guarantees the harness rests on over a wide input
//! space: that normalization is stable (a snapshot always matches itself), and
//! that the diff is a faithful edit script (its `+`/` ` lines reconstruct the
//! actual text, its `-`/` ` lines reconstruct the expected text).

use proptest::prelude::*;
use test_lang::{Change, Diff, Snapshot};

/// Reconstruct one side of a diff by keeping the lines that belong to it.
///
/// Keeping [`Equal`](Change::Equal) + [`Delete`](Change::Delete) lines rebuilds
/// the expected text; keeping `Equal` + [`Insert`](Change::Insert) rebuilds the
/// actual text. A correct edit script must round-trip both.
fn reconstruct(diff: &Diff, keep: impl Fn(Change) -> bool) -> String {
    let kept: Vec<&str> = diff
        .changes()
        .filter(|(change, _)| keep(*change))
        .map(|(_, line)| line)
        .collect();
    kept.join("\n")
}

proptest! {
    /// A snapshot always checks clean against the exact text it captured.
    #[test]
    fn prop_snapshot_matches_its_own_text(input in ".*") {
        let snapshot = Snapshot::new(&input);
        prop_assert!(snapshot.check(snapshot.as_str()).is_ok());
    }

    /// Normalization is idempotent: re-wrapping a snapshot's own text yields an
    /// identical snapshot, so there is no hidden second-pass difference.
    #[test]
    fn prop_normalization_is_idempotent(input in ".*") {
        let once = Snapshot::new(&input);
        let twice = Snapshot::new(once.as_str());
        prop_assert_eq!(once.as_str(), twice.as_str());
    }

    /// `check` succeeds exactly when the two texts normalize to the same thing.
    #[test]
    fn prop_check_ok_iff_normalized_equal(a in ".*", b in ".*") {
        let sa = Snapshot::new(&a);
        let sb = Snapshot::new(&b);
        let normalized_equal = sa.as_str() == sb.as_str();
        prop_assert_eq!(sa.check(sb.as_str()).is_ok(), normalized_equal);
    }

    /// The diff's kept-actual reconstruction equals the actual text, and its
    /// kept-expected reconstruction equals the expected text.
    #[test]
    fn prop_diff_reconstructs_both_sides(
        expected in "[a-c\n]{0,40}",
        actual in "[a-c\n]{0,40}",
    ) {
        // Compare in normalized space, since that is what the diff operates on.
        let expected = Snapshot::new(&expected);
        let actual = Snapshot::new(&actual);
        let diff = Diff::lines(expected.as_str(), actual.as_str());

        let rebuilt_expected = reconstruct(&diff, |c| c != Change::Insert);
        let rebuilt_actual = reconstruct(&diff, |c| c != Change::Delete);

        prop_assert_eq!(rebuilt_expected, expected.as_str());
        prop_assert_eq!(rebuilt_actual, actual.as_str());
    }

    /// A diff is empty if and only if the two normalized inputs are equal.
    #[test]
    fn prop_diff_empty_iff_equal(
        expected in "[a-c\n]{0,40}",
        actual in "[a-c\n]{0,40}",
    ) {
        let expected = Snapshot::new(&expected);
        let actual = Snapshot::new(&actual);
        let diff = Diff::lines(expected.as_str(), actual.as_str());
        prop_assert_eq!(diff.is_empty(), expected.as_str() == actual.as_str());
    }

    /// `per_line` over displayable items equals joining those items with `\n`
    /// (after normalization), for any set of newline-free lexemes.
    #[test]
    fn prop_per_line_equals_newline_join(items in prop::collection::vec("[a-z]{1,6}", 0..20)) {
        let snapshot = Snapshot::per_line(&items);
        prop_assert!(snapshot.check(items.join("\n")).is_ok());
    }
}
