# ISK ROADMAP

**Permanent mission: Build ISK for agents, not merely with agents.** Design
the language and development model around how autonomous agents understand,
construct, transform, and verify software. Human-readable source remains a
view of the program; checked semantics give agents a way to query what matters,
make validated changes, and verify their impact.

Measure progress by the total effort to deliver correct, verified,
maintainable software while preserving safety, determinism, and native
performance. The Rust bootstrap and numerical pilot serve that mission;
neither defines the language. The five
[engineering pillars](docs/architecture.md) evolve together. Apply the
[architecture decision gates](docs/architecture.md#decision-and-release-gates)
at every significant milestone.

## Current baseline

- A small executable compiler supports `check`, `build`, `run`, targeted
  function inspection, and rename previews. It uses Rust as its bootstrap backend.
- The current syntax and semantics are provisional. See the
  [bootstrap ADR](docs/adr/0001-mvp-language-subset.md) for the limits.
- The [design evaluation](docs/research/design-evaluation.md) defines
  competing hypotheses and a benchmark protocol. No comparative results
  have been collected yet.

## 1. Stabilize the bootstrap

Keep this milestone focused on the current language subset. Cover the CLI's
success and failure paths, including rejected source and runtime arithmetic
failures. Make it easy to run the example and repeat the same checks locally.

**Done when:** formatting, lint, and tests pass in CI, the example runs as
documented, and integration tests distinguish compiler diagnostics from
runtime errors. Track the diagnostic code, location, and first verification
failure for the benchmark tasks so later versions can compare repair effort.

## 2. Run a reproducible benchmark pilot

The reference runner and shared GCD fixtures establish implementation and
modification tasks. Controlled agent trials and their raw results are still open.

Start with one numerical task that all three languages can express: ISK, Rust,
and Python. Use equivalent requirements and hidden tests and hold the model
configuration, tool access, and limits constant across trial groups. Record
correctness on the first and final attempts, time to verified change, context
and total tokens, failed builds and tests, correction iterations, tool calls,
and cost per success. Record compiler and runtime measurements separately.

**Done when:** the harness, fixtures, and raw results conforming to the
[result schema](benchmarks/result.schema.json) are published, and repeated
trials report variability and limitations. Treat the pilot as a check of the
method, not a language-wide conclusion.

## 3. Prototype a validated semantic edit

The single-file rename prototype uses typed function signatures and source-order
IDs. An ID is deterministic for identical inputs and survives renaming, but
can change when declarations are added, removed, or reordered.

`CheckedProgram` shares resolved semantic information between inspection and
rename without requiring the CLI or backend. Rename revalidates the candidate
and never writes the input. More durable IDs and transactional writes are
future work, not requirements imposed on this prototype.

**Done when:** tests show deterministic identities for identical inputs,
successful renames, unchanged unrelated source, and rollback on invalid
edits. Compare the operation with a text-based rename on the same task.
For the semantic CLI milestone, compare success rate, context tokens, and
correction iterations with plain-text workflows. Keep untouched source and
deterministic diagnostics as hard requirements; set quantitative targets in
the experiment before running the comparison.

## 4. Demonstrate a complete agent-native workflow

Start with a small single-file project and a behavior-preserving rename of a
recursive helper with multiple callers and unrelated same-name bindings.
Have the agent query symbols and dependencies, retrieve the context it
needs, preview and validate a structured change, apply it, run affected
tests (or the full suite when impact is uncertain), and report the direct
callers, test scope, and result.

**Done when:** the workflow runs end to end, invalid or stale plans leave
the source untouched, and an independently repeated
[semantic-vs-text experiment](docs/research/design-evaluation.md#complete-agent-workflow-experiment)
reports context, total tokens, corrections, correctness, and time. Decide
whether to keep, revise, or reject the approach based on that evidence.

## After the pilot

Evaluate syntax, inference, error handling, ownership, concurrency, effects,
and contracts in focused experiments rather than treating the bootstrap as
the default design. Adopt a candidate in an ADR only after recording its
results, trade-offs, and limitations in the
[design evaluation](docs/research/design-evaluation.md).
Evaluate alternate backends and eventual self-hosting only when they serve
the language or demonstrated agent workflows.
