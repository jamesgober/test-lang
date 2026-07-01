//! Integration tests exercising the snapshot harness end to end, the way a
//! language front-end would use it: source in, rendered stage output asserted
//! against an expected block of text.

use test_lang::{Change, Snapshot};

/// A stand-in lexer: splits on whitespace and tags each lexeme with a kind, the
/// way a real token stream would render. This keeps the tests self-contained
/// while mirroring the shape of the crates test-lang is built to snapshot.
fn lex(source: &str) -> Vec<String> {
    source
        .split_whitespace()
        .map(|lexeme| {
            let kind = if lexeme.chars().all(|c| c.is_ascii_digit()) {
                "Int"
            } else if lexeme == "=" {
                "Eq"
            } else {
                "Ident"
            };
            format!("{kind}({lexeme})")
        })
        .collect()
}

#[test]
fn test_token_stream_snapshot_matches() {
    let snapshot = Snapshot::per_line(lex("let x = 1"));
    snapshot
        .check("Ident(let)\nIdent(x)\nEq(=)\nInt(1)")
        .expect("token stream should match the expected snapshot");
}

#[test]
fn test_token_stream_snapshot_reports_drift() {
    let snapshot = Snapshot::per_line(lex("let y = 1"));
    let err = snapshot
        .check("Ident(let)\nIdent(x)\nEq(=)\nInt(1)")
        .expect_err("a changed identifier should fail the check");

    let rendered = err.to_string();
    assert!(
        rendered.contains("-Ident(x)"),
        "expected line missing: {rendered}"
    );
    assert!(
        rendered.contains("+Ident(y)"),
        "actual line present: {rendered}"
    );
}

#[test]
fn test_ast_debug_snapshot() {
    // Fields are read through the derived `Debug` the snapshot captures; the
    // dead-code lint does not count that as a use.
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Binary {
        op: char,
        lhs: i64,
        rhs: i64,
    }

    let tree = Binary {
        op: '+',
        lhs: 1,
        rhs: 2,
    };
    let snapshot = Snapshot::debug(&tree);

    // The pretty-printed tree is stable across platforms after normalization.
    assert!(snapshot.check(snapshot.as_str()).is_ok());
    assert!(snapshot.as_str().contains("op: '+'"));
    assert!(snapshot.as_str().contains("lhs: 1"));
}

#[test]
fn test_diagnostics_snapshot() {
    // A rendered diagnostic, the kind a diagnostics layer would produce.
    let rendered =
        "error: unexpected token\n  --> 1:5\n  |\n1 | let = 1\n  |     ^ expected identifier";
    let snapshot = Snapshot::new(rendered);

    snapshot
        .check(rendered)
        .expect("a diagnostic should snapshot verbatim");
}

#[test]
fn test_cross_platform_line_endings_and_trailing_space() {
    // Simulate output captured on Windows: CRLF endings, a formatter's trailing
    // space. It must still match a hand-written LF expectation.
    let windows_output = "error: bad token  \r\n  --> 1:1\r\n";
    let snapshot = Snapshot::new(windows_output);

    snapshot
        .check("error: bad token\n  --> 1:1")
        .expect("normalization should erase platform differences");
}

#[test]
fn test_empty_output_snapshots_clean() {
    let snapshot = Snapshot::per_line(Vec::<String>::new());
    snapshot
        .check("")
        .expect("no tokens means an empty snapshot");
}

#[test]
fn test_multiline_insertion_reports_each_line() {
    let snapshot = Snapshot::per_line(["a", "b", "c", "d"]);
    let err = snapshot.check("a\nd").unwrap_err();

    let inserted: Vec<_> = err
        .diff()
        .changes()
        .filter(|(change, _)| *change == Change::Insert)
        .map(|(_, line)| line.to_string())
        .collect();
    assert_eq!(inserted, ["b", "c"]);
}

#[test]
fn test_display_value_snapshot() {
    struct Version(u32, u32, u32);
    impl std::fmt::Display for Version {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}.{}.{}", self.0, self.1, self.2)
        }
    }

    Snapshot::display(&Version(0, 2, 0))
        .check("0.2.0")
        .expect("a Display value should snapshot to its rendering");
}
