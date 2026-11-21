# ISK roadmap

ISK aims to reduce the total cost for AI agents to produce correct,
maintainable software without sacrificing safety or human readability.
Fewer tokens alone do not establish an improvement.

## Current baseline

- A small executable compiler supports `check`, `build`, and `run` and
  uses Rust as its bootstrap backend.
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
runtime errors.

## 2. Run a reproducible benchmark pilot

Start with one numerical task that all three languages can express: ISK, Rust,
and Python. Use equivalent requirements and hidden tests and hold the model
configuration, tool access, and limits constant across trial groups. Record
completion, tokens, failed builds, correction iterations, and tool calls.
Record compiler and runtime measurements separately from agent outcomes.

**Done when:** the harness, fixtures, and raw results conforming to the
[result schema](benchmarks/result.schema.json) are published, and repeated
trials report variability and limitations. Treat the pilot as a check of the
method, not a language-wide conclusion.

## 3. Prototype a validated semantic edit

Introduce a typed representation with stable symbol IDs. Start with one
operation: rename a function and its resolved references. Revalidate the
result before writing anything, and leave the original source untouched if
the rename conflicts or fails validation.

**Done when:** tests show deterministic identities for identical inputs,
successful renames, unchanged unrelated source, and rollback on invalid
edits. Compare the operation with a text-based rename on the same task.

## After the pilot

Evaluate syntax, inference, error handling, ownership, concurrency, effects,
and contracts in focused experiments rather than treating the bootstrap as
the default design. Adopt a candidate in an ADR only after recording its
results, trade-offs, and limitations in the
[design evaluation](docs/research/design-evaluation.md).
