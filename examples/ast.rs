//! Snapshotting a syntax tree.
//!
//! Most AST node types derive `Debug` but not `Display`. `Snapshot::debug`
//! captures the pretty-printed (`{:#?}`) tree, so a parser test can assert the
//! whole shape at once and get a line-level diff when the structure changes.
//!
//! Run with `cargo run --example ast`.

use test_lang::Snapshot;

/// An expression node, the sort a recursive-descent parser would build.
///
/// The variants' payloads are read through the derived `Debug` that the
/// snapshot captures, which the dead-code lint does not count as a use.
#[derive(Debug)]
#[allow(dead_code)]
enum Expr {
    Int(i64),
    Binary {
        op: char,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
}

/// Parse `1 + 2 * 3` into a tree with the usual precedence. Hard-coded here to
/// keep the example about snapshotting, not parsing.
fn parse_example() -> Expr {
    Expr::Binary {
        op: '+',
        lhs: Box::new(Expr::Int(1)),
        rhs: Box::new(Expr::Binary {
            op: '*',
            lhs: Box::new(Expr::Int(2)),
            rhs: Box::new(Expr::Int(3)),
        }),
    }
}

fn main() {
    let tree = parse_example();
    let snapshot = Snapshot::debug(&tree);

    println!("captured syntax tree:\n{snapshot}\n");

    // A snapshot always matches the text it captured — accept it by pasting the
    // rendering into the test as the expected value.
    match snapshot.check(snapshot.as_str()) {
        Ok(()) => println!("tree snapshot is stable"),
        Err(mismatch) => {
            println!("{mismatch}");
            std::process::exit(1);
        }
    }
}
