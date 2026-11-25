# ISK ROADMAP

**Permanent mission: Build ISK for agents, not merely with agents.** Design
the language and development model around how autonomous agents understand,
construct, transform, and verify software. Human-readable source remains a
view of the program; checked semantics give agents a way to query what matters,
make validated changes, and verify their impact.

Measure progress by the total effort to deliver correct, verified,
maintainable software while preserving safety, determinism, and native
performance. The Rust bootstrap and numerical pilot serve that mission;
neither defines the language. The seven
[engineering pillars](docs/architecture.md) evolve together. Apply the
[architecture decision gates](docs/architecture.md#decision-and-release-gates)
at every significant milestone.

## Next research milestones, in priority order

Carry the [vision](docs/vision.md) through all seven pillars: A language,
B representation, C agent interfaces, D verification, E libraries, F evidence
and G performance/safety. Do not wait for more CLI infrastructure to start
milestone 3. Each milestone ends with a decision, not just an implementation.

1. **Fairer rename (B/C/F).** The [first multi-caller run](benchmarks/results/2026-11-22-parcel/README.md)
   gives both arms search, range reads and real test commands. Replicate
   the [protocol](benchmarks/controller-trials/parcel-protocol.md) on a new
   model and held-out fixture before generalizing.
2. **One checked edit (B/C).** Use replication traces to decide whether
   parameter rename addresses a recurring context or edit cost. If so, prototype
   it and compare with targeted text edits; test same-name bindings in other
   functions, stale snapshots and conflicts. Stop if locality or rollback fails.
3. **One language distinction (A/F/G).** In the [controller experiment](docs/research/agent-native-language-design.md#state-machine-case-keep-behavior-fixed)
   compare numeric IDs plus a lint with distinct state/event types, holding
   spelling and tooling as constant as possible. Check swapped inputs,
   invalid-ID handling, native cost and repair effort before proposing syntax.
4. **Verification (D/F).** On seeded body changes compare focused-first
   versus full-first explicit tests. Record missed failures, total verified
   time and corrections; both arms finish with the same full suite.
5. **Larger application (A/B/D/E/G).** Start with the [host-backed REST
   controller](https://github.com/deed-ee/isk/issues/23): the host handles HTTP
   and JSON, while ISK owns the transition. Use identical acceptance tests
   for an equivalent reference service and report host costs separately.
   Later, extract a local policy library and change its interface and consumer
   together to measure discovery and integration. Neither slice needs a registry.

## Current baseline

- A small executable compiler supports `check`, `build`, `run`, targeted
  function inspection, and rename previews. It uses Rust as its bootstrap backend.
- The current syntax and semantics are provisional. See the
  [bootstrap ADR](docs/adr/0001-mvp-language-subset.md) for the limits.
- The [design evaluation](docs/research/design-evaluation.md) defines
  competing hypotheses and a benchmark protocol. The [first controller
  pilot](benchmarks/results/2026-11-22-controller/README.md) compares tool
  access within ISK. Cross-language agent trials remain open.

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
modification tasks. Controlled cross-language numerical trials and their raw
results are still open; the within-ISK controller pilot is separate.

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
verification. Named boolean tests can run via `isk test`, with the full suite as the
fallback. The controller pilot measures a simpler rename; the recursive
workflow with multiple callers, test impact and context attribution remains open.

**Done when:** the workflow runs end to end, invalid or stale plans leave
the source untouched, and an independently repeated
[semantic-vs-text experiment](docs/research/design-evaluation.md#complete-agent-workflow-experiment)
reports context, total tokens, corrections, correctness, and time. Decide
whether to keep, revise, or reject the approach based on that evidence.

## 5. Grow a reference application

The [controller showcase](showcase/README.md) starts with a tested,
deterministic state machine. Evaluate the host-backed REST slice first, then
revisit event-driven modules, MQTT and larger concurrent systems as
language capabilities are validated. These stages guide experiments; they
do not require networking or advanced syntax now.

**Done for Stage A when:** the controller and supported examples execute in
normal CI, and an independent evaluation uses the
[controller maintenance tasks](benchmarks/controller-tasks.md) to measure
repair, extension, and refactoring effort. Use those results to identify
missing capabilities and record trade-offs before adopting language changes.

## 6. Shorten the verification feedback loop

The single-file selector suggests test roots from resolved calls. A separate
[prototype runner](docs/cli.md#run-explicit-tests-experimental) executes
explicitly named roots and reports results linked to checked symbols. Neither
discovers tests. Evaluate coverage, properties, and caching only after the
simpler workflow is measured. Keep the full regression suite in CI regardless
of local test selection.

**Done when:** seeded-change experiments compare focused-first and full-first
verification on the same ISK revision, report affected tests missed against
the full suite, and measure end-to-end time, test time, tokens, and correction
cycles without weakening final verification. Separately compare equivalent
development workflows in Rust and Python.

## 7. Share semantic operations across interfaces

The CLI uses the shared checked snapshot for inspection, rename, and
affected-test suggestions. `check`, `inspect` and `affected-tests` expose
versioned JSON response formats. Prototype the
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
`src/main.isk` without configuration. `check`, `build`, `run`, and `inspect`
resolve a single entry from an explicit root, optionally configured by a
minimal manifest as shown by the controller. Evaluate the manifest against
the convention-only baseline before extending it. Treat project discovery
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
