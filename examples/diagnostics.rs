//! Snapshotting a rendered diagnostic.
//!
//! A diagnostics layer produces multi-line, caret-annotated output — exactly
//! the kind of thing that is painful to assert field by field and easy to
//! assert as a snapshot. Normalization means the same expected block passes
//! whether the diagnostic was rendered on Windows or Linux.
//!
//! Run with `cargo run --example diagnostics`.

use test_lang::Snapshot;

/// Render a diagnostic for an unexpected `=` where an identifier was required.
/// A real diagnostics crate would compute the caret span from the source; the
/// shape of the output is what matters for the snapshot.
fn render_diagnostic(source: &str, caret_col: usize) -> String {
    let underline = format!("{}^ expected identifier", " ".repeat(caret_col));
    format!(
        "error: unexpected token\n  --> 1:{}\n  |\n1 | {source}\n  | {underline}",
        caret_col + 1,
    )
}

fn main() {
    let rendered = render_diagnostic("let = 1", 4);
    let snapshot = Snapshot::new(&rendered);

    println!("captured diagnostic:\n{snapshot}\n");

    let expected = "\
error: unexpected token
  --> 1:5
  |
1 | let = 1
  |     ^ expected identifier";

    match snapshot.check(expected) {
        Ok(()) => println!("diagnostic snapshot matches"),
        Err(mismatch) => {
            println!("{mismatch}");
            std::process::exit(1);
        }
    }
}
