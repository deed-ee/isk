# ADR 0001: MVP language subset and compiler architecture

- Status: accepted
- Date: 2026-11-21

## Context

The repository is an empty scaffold. The first milestone must prove an end-to-end
source-to-native-executable path without locking the language into advanced,
unmeasured runtime or ownership decisions.

## Decisions

- Bootstrap as a Cargo workspace with one compiler package. Keep lexer, parser,
  semantic checking, and Rust generation as distinct internal stages; split
  crates only when stable APIs establish useful dependency boundaries.
- Use explicit `fn`, parameter and return types, `let`/`mut`, semicolon-terminated
  bindings, and final expressions as block results. This favors predictable,
  locally parseable structure over implicit indentation or compact punctuation.
- Support `u64`, `bool`, function calls, arithmetic, comparisons, `if` expressions,
  local bindings, and `print` for the first executable slice.
- Compile integers as Rust `u64` with explicit checked arithmetic. Overflow and
  division by zero panic at runtime in this MVP; behavior does not vary by
  Rust build profile. A future result/error design may replace panics.
- Use Rust's ownership and memory safety for generated programs; ISK currently
  has no reference, heap collection, concurrency, or independent ownership model.
- Generate deterministic, readable, safe Rust and invoke stable `rustc`.
  Diagnostics have stable code identifiers and source line/column positions;
  rustc diagnostics for generated code do not yet map back to ISK spans.

## Alternatives considered

- Implicitly typed, punctuation-light bindings are compact but make the first
  checker and error reporting less explicit. The MVP uses named `let` bindings
  and optional annotations instead.
- A mandatory tracing garbage collector would simplify ownership but introduce
  runtime and performance costs without measurements. It is deferred; the MVP
  inherits Rust ownership behavior through its backend.
- Separate crates for every conceptual stage would establish boundaries before
  their interfaces are known. A single package with staged modules is preferred
  until those boundaries can be justified by implementation experience.

## Consequences

The language is intentionally small and not yet a safe-Rust equivalent. This
compiler establishes a testable vertical slice, not completion of the broader
language, agent-editing, or efficiency-benchmark roadmap.
