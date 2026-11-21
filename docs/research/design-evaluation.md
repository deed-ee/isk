# ISK design evaluation: candidates and evidence gates

ISK must be designed for agent-authored software rather than inherit a familiar
language's conventions by default. Rust is the implementation language and
initial native backend required for the bootstrap; it is not ISK's design
template. The current compiler is an executable baseline, not a decision that
ISK source should look like Rust or that Rust ownership is ISK ownership.

## Status

All design choices below are **open**. The first compiler slice is not evidence
that a candidate is clearer, safer, faster, or cheaper for agents. No
agent-efficiency or comparative-language results have been collected yet.
Do not mark a candidate accepted until its experiment and limitations are
recorded in an ADR.

## Candidates and evaluation criteria

| Area | Candidate approaches to compare | Evidence required before selection |
| --- | --- | --- |
| Syntax and grammar | Explicit delimiter blocks; indentation/layout blocks; declaration- or schema-oriented forms for common data transformations | Blinded agent completion, correction iterations, tool calls, model-token counts with the tokenizer named, parse/diagnostic error rates, human review |
| Ownership and memory | Inferred affine/ownership rules; region-based allocation; reference counting; tracing collection; explicit per-type policies | Safety/property tests, aliasing and lifetime task success, runtime and peak-memory workloads, binary size, allocation behavior |
| Error handling | Explicit typed result values; inferred checked effects; algebraic error unions; exception-like propagation | Error-path task success, unhandled-error detection, diagnostic iterations, runtime overhead, generated-code inspection |
| Type inference | Bidirectional local inference; constrained global inference; explicit annotations at public boundaries | Annotation burden, ambiguity and error-localization rates, agent correction cost, checker time, readability review |
| Concurrency | Structured task scopes; actor/message passing; ownership-checked shared state; explicit async effects | Race-freedom checks, cancellation/resource-leak tests, task-completion and repair rates, throughput, latency, memory |
| Semantic representation | Typed HIR with stable symbol IDs; dependency/effect graphs; source-preserving semantic nodes | Incremental invalidation correctness, deterministic identity, affected-context size, edit/rollback correctness |
| Structured editing | Validated transactional semantic operations; patch-based tree edits; source-aware edits | Change locality, preservation of comments/formatting, validation failures, rollback behavior, tokens and tool calls per change |
| Compilation and verification | Batch compilation; incremental typed stages; contracts and property checks; differential/reference execution | Clean and incremental build time, diagnostic stability, detected-bug rate, false-positive rate, reproducibility |

These approaches are hypotheses, not a ranking. Hybrid designs are possible.
Safety, human inspectability, and predictable diagnostics are hard constraints;
token count alone must never choose a design.

## Benchmark protocol

1. Implement equivalent task suites in ISK and baselines (initially Rust and
   Python): numerical computation, JSON processing, file processing,
   concurrent tasks, a small HTTP service, and stateful business logic.
2. Keep functional requirements, hidden tests, task context, tool access, model,
   model version, sampling settings, and time limits identical within a trial
   group. Randomize candidate order and repeat agent tasks; report completion
   rates and variability, not just averages.
3. Capture input/output model tokens using the named model tokenizer, agent
   tool calls, failed builds, correction iterations, wall time, completion,
   runtime, peak memory, and binary size. Separate authoring measurements from
   compiler/runtime measurements.
4. Record compiler, runtime, OS, CPU, memory, benchmark revision, prompts,
   model configuration, and raw per-trial results. Use the machine-readable
   [`../../benchmarks/result.schema.json`](../../benchmarks/result.schema.json)
   format. Missing measurements are `null`; do not estimate or fabricate them.
5. Run deterministic compiler/runtime measurements repeatedly on the same
   hardware with warm-up and report median and spread. Measure agent outcomes
   separately from microbenchmarks so backend/tooling effects are visible.
6. Publish the exact harness and task fixtures before using results to accept a
   design. Re-run after material compiler, model, or task-suite changes.

The initial pilot should establish variance and harness correctness; it is not
sufficient evidence for a language-wide conclusion. Design-specific trials
should vary one factor at a time where feasible. For example, compare grammar
forms without changing memory semantics, then run a separate memory-policy
experiment with identical syntax and tasks.

## Current evidence

The bootstrap integration test executes one factorial program and tests
overflow behavior. This validates only that a Rust-backed vertical slice runs;
it does not compare language designs, measure agent effort, or establish
performance advantages. There are no comparative benchmark results to report.
