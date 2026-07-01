# test-lang - Roadmap

> Path from scaffold to a stable 1.0. Hard parts are front-loaded; each phase has hard exit criteria.
> Master plan: ../../_strategy/LANG_COLLECTION.md
>
> **Anti-deferral rule:** no listed hard task moves to a later phase unless this file records the move and the reason.

## v0.1.0 - Scaffold (DONE)
Compiles, CI green, structure correct, no domain logic.
- [x] Manifest, README, CHANGELOG, REPS, dual license, CI, deny, clippy, rustfmt.

## v0.2.0 - Core (THE HARD PART, NOT DEFERRED) (DONE)
A snapshot harness: given source, assert the token stream / AST / diagnostics.
Exit criteria:
- [x] Every public item has rustdoc + a runnable example.
- [x] Core invariants property-tested (`tests/proptests.rs`); API authored (`docs/API.md`).

**Scope decision — concrete lexer/ast/parser/diag deps not wired.** The original
plan wired those four crates "when first used." They are not used: the harness
operates on `core::fmt::Display`/`Debug`, so it snapshots any front-end output
without a compile-time dependency on the crate that produced it. Taking concrete
deps would violate the REPS architecture rule that cross-crate coupling flow
through abstractions, not concrete internals, and would couple test-lang's
release cadence to four other crates for no functional gain. Recorded here per
the anti-deferral rule: this is a design change, not a deferral — the task is
dropped, not moved to a later phase.

## v1.0.0 - API freeze
Public surface stable and frozen until 2.0.
- [ ] docs/API.md marked stable; SemVer promise recorded.
- [ ] Full test + benchmark suite green on all three platforms.
