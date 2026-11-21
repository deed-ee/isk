# ADR 0001: Bootstrap slice (provisional)

- Status: provisional; not a ratified language-design decision
- Date: 2026-11-21

## Context

The repository began as an empty scaffold. A narrow, executable bootstrap slice
is useful for testing the toolchain, but implementation convenience is not
evidence that its syntax or semantics are optimal for agent-authored programs.
ISK is not intended to be a simplified Rust dialect or a thin Rust transpiler.
The bootstrap does not ratify a text-first editing model, a permanent program
representation, or Rust-specific ownership or compilation semantics. See the
[semantic-first architecture](../architecture.md).

## Temporary implementation baseline

The current compiler accepts a small language with explicit `fn`, typed
parameters, `let`/`mut`, arithmetic, conditionals, calls, and a `print` builtin.
It checks a subset of types, emits Rust, and invokes stable `rustc`. This is a
vertical-slice experiment only. Its Rust-like syntax, panic-based arithmetic
failure, lack of independent ownership semantics, and direct source generation
are limitations to evaluate—not endorsed final language choices.

No syntax, ownership, error, inference, concurrency, semantic-editing, or
verification design is ratified by this ADR. The alternatives and required
experiments are tracked in
[`../research/design-evaluation.md`](../research/design-evaluation.md). Promote
a candidate to an accepted ADR only after the relevant benchmark has been run,
its setup and results recorded, and its limitations reviewed.

The bootstrap enables comparisons but has no demonstrated agent-efficiency
advantage. Future ADRs must record an agent-efficiency hypothesis, the
comparison baseline, and acceptance criteria using the [ADR template](template.md).
