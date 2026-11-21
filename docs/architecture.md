# ISK engineering architecture

If programming languages were invented primarily for agents, text would not
have to be their only interface. ISK treats human-readable source as one
representation of a program and checked semantics as the basis for understanding,
editing, and verifying it. Five pillars evolve together:

1. Agent-native language design.
2. Semantic-first program representation.
3. Compiler-integrated agent development interfaces.
4. Agent development efficiency and empirical evaluation.
5. Native performance, safety, and correctness.

## What the bootstrap proves and where it stops

Rust is used to implement and bootstrap ISK, not to define its syntax or
semantics. The current pipeline lexes and parses source, checks signatures and
expressions, then emits Rust. The checker already records typed function
signatures, source-order function IDs, and resolved call locations. Those IDs
are deterministic for the same declaration order, not durable across
structural edits. `rename` uses the checked information, preserves untouched
source, and revalidates the candidate before printing it.

`CheckedProgram` reuses one parsed and checked source snapshot for read-only
queries and validated rename previews. `inspect` exposes one signature and
its resolved call sites, not function bodies. This proves the API boundary,
not the final representation: expressions still lower from the AST, IDs and
byte spans are snapshot-local, diagnostics are rendered strings, and a preview
does not protect an on-disk write against stale input. There is no module
graph, incremental checking, or test impact analysis yet.

## Incremental evolution

1. Extend the snapshot with caller relationships and structured diagnostics.
   Keep identity separate from source spans; specify how IDs survive rename,
   insertion, and reordering before exposing cross-snapshot identities.
2. Add a validated change plan tied to the original snapshot. Reject stale
   inputs and invalid or conflicting edits before an atomic single-file write.
   Map changed symbols to declared test targets, falling back to the full
   suite when the mapping cannot safely narrow it.
3. Introduce typed HIR when a query or verification task needs it. Keep AST
   and human-readable source projections separate from resolved semantic facts.
   Extend those facts with types, module dependencies, control/data flow,
   contracts, effects, error propagation, ownership and lifetimes, invariants,
   and concurrency as experiments justify them. Add incremental verification
   with full-check equivalence tests before relying on cached results.

CLI and any later MCP adapter consume the same semantic API. Native code
generation is a consumer of language semantics, not their definition. Keep
the Rust backend while it supports experiments; evaluate alternative lowering
only when the language or performance needs it.

## Decision and release gates

For each significant change, record the agent-efficiency hypothesis, the prior
ISK version or relevant alternative, and a measurable success criterion before
evaluation. Record correctness, maintainability, safety, native performance,
and determinism constraints separately. After the smallest useful
implementation, record measurements, variability, limitations, and the
decision to keep, revise, or reject it. Use the
[ADR template](adr/template.md) and the [evaluation protocol](../benchmarks/README.md).
Run correctness and short reference checks continuously; reserve expensive
controlled agent trials for milestones. Do not ratify a language design on
implementation convenience or unmeasured benefits.
