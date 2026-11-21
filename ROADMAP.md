# ISK ROADMAP

**Permanent mission: Build ISK for agents, not merely with agents.** Design
the language and development model around how autonomous agents understand,
construct, transform, and verify software. Human-readable source remains a
view of the program; checked semantics give agents a way to query what matters,
make validated changes, and verify their impact.

Measure progress by the total effort to deliver correct, verified,
maintainable software while preserving safety, determinism, and native
performance. The Rust bootstrap and numerical pilot serve that mission;
neither defines the language. The eight
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
rename without requiring the CLI or backend. Rename revalidates the candidate;
the default mode never writes the input. More durable IDs remain future work.

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

The CLI now supports this sequence for a single file: inspect direct callers,
print a checked rename plan, apply it to the same source snapshot, and run
verification. Test execution remains manual, with the full suite as the
fallback; the independent agent comparison has not been run.

**Done when:** the workflow runs end to end, invalid or stale plans leave
the source untouched, and an independently repeated
[semantic-vs-text experiment](docs/research/design-evaluation.md#complete-agent-workflow-experiment)
reports context, total tokens, corrections, correctness, and time. Decide
whether to keep, revise, or reject the approach based on that evidence.

## 5. Grow a reference application

The [controller showcase](showcase/README.md) starts with a tested,
deterministic state machine. Grow it through event-driven modules, an MQTT
service, a REST microservice, and eventually a larger concurrent system as
language capabilities are validated. These stages guide experiments; they
do not require networking or advanced syntax now.

**Done for Stage A when:** the controller and supported examples execute in
normal CI, and an independent evaluation uses the
[controller maintenance tasks](benchmarks/controller-tasks.md) to measure
repair, extension, and refactoring effort. Use those results to identify
missing capabilities and record trade-offs before adopting language changes.

## 6. Shorten the verification feedback loop

The single-file selector suggests test roots from resolved calls; it does not
discover or run tests. Next, prototype deterministic test execution and
structured failure reports linked to symbols. Evaluate coverage, properties,
and caching only after the simpler workflow is measured. Keep the full
regression suite in CI regardless of local test selection.

**Done when:** seeded-change experiments compare focused-first and full-first
verification on the same ISK revision, report affected tests missed against
the full suite, and measure end-to-end time, test time, tokens, and correction
cycles without weakening final verification. Separately compare equivalent
development workflows in Rust and Python.

## 7. Share semantic operations across interfaces

The CLI uses the shared checked snapshot for inspection, rename, and
affected-test suggestions. JSON for `check` and `inspect` exposes the first
versioned response formats. Prototype the
[read-only MCP adapter](docs/architecture.md#minimal-mcp-adapter-design)
only over working operations; require parity with CLI results before
expanding it. Compare text-only, semantic CLI, and MCP workflows on the same
ISK revision, then compare equivalent Rust and Python tasks separately.
Report tokens, context, tool calls, time, corrections, success, and cost per
verified change. Do not introduce persistence until repeated parsing is
measured as a bottleneck.

## 8. Make software reuse agent-native

Start with the controller consuming a local library. The
[package architecture](docs/packages.md) stages the work: local modules and
libraries, deterministic multi-package workspaces, locked versioned packages,
secure distribution and provenance, then semantic discovery and safe updates.
No public registry is needed for the first stages.

**Done for the first slice when:** a checked public interface can be queried
without reading private bodies, local resolution is repeatable, and the
controller and visibility tests pass. Compare agent effort against source and
documentation lookup before committing to module syntax or a package format.
Advance later stages only when their integrity and isolation boundaries can
be verified.

## 9. Make application projects predictable

The first [project scaffold](docs/projects.md) creates a runnable
`src/main.isk` without configuration. The controller demonstrates this
layout. Next, evaluate a minimal manifest and explicit project loading
before adding multi-module or workspace features. Treat project discovery
and verification as first-class parts of the shared semantic API.

**Done for the first project-aware slice when:** the CLI can check and run
an explicitly selected application root reproducibly, diagnose invalid
entries, and expose its checked entry point without reading unrelated files.
Compare scaffolding and project comprehension with equivalent Rust and
Python workflows and retain full regression coverage in CI.

## After the pilot

Evaluate syntax, inference, error handling, ownership, concurrency, effects,
and contracts in focused experiments rather than treating the bootstrap as
the default design. Adopt a candidate in an ADR only after recording its
results, trade-offs, and limitations in the
[design evaluation](docs/research/design-evaluation.md).
Evaluate alternate backends and eventual self-hosting only when they serve
the language or demonstrated agent workflows.
