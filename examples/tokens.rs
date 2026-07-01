//! Snapshotting a token stream.
//!
//! The most common use of the harness: run source through a lexer and assert
//! the token stream, one token per line, so a failure points at the exact token
//! that changed.
//!
//! Run with `cargo run --example tokens`.

use test_lang::Snapshot;

/// A token: its kind and the slice of source it covers. A real lexer would
/// carry a span too; here the kind and text are enough to render.
#[derive(Debug)]
struct Token {
    kind: Kind,
    text: String,
}

#[derive(Debug)]
enum Kind {
    Ident,
    Int,
    Eq,
    Plus,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}({})", self.kind, self.text)
    }
}

/// A minimal whitespace lexer, standing in for the real one under test.
fn lex(source: &str) -> Vec<Token> {
    source
        .split_whitespace()
        .map(|lexeme| {
            let kind = match lexeme {
                "=" => Kind::Eq,
                "+" => Kind::Plus,
                _ if lexeme.chars().all(|c| c.is_ascii_digit()) => Kind::Int,
                _ => Kind::Ident,
            };
            Token {
                kind,
                text: lexeme.to_string(),
            }
        })
        .collect()
}

fn main() {
    let tokens = lex("total = a + 42");
    let snapshot = Snapshot::per_line(&tokens);

    println!("captured token stream:\n{snapshot}\n");

    let expected = "\
Ident(total)
Eq(=)
Ident(a)
Plus(+)
Int(42)";

    match snapshot.check(expected) {
        Ok(()) => println!("snapshot matches expected token stream"),
        Err(mismatch) => {
            println!("{mismatch}");
            std::process::exit(1);
        }
    }
}
