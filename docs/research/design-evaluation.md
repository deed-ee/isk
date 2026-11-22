# ISK design evaluation: candidates and evidence gates

ISK must be designed for agent-authored software rather than inherit a familiar
language's conventions by default. Rust is the implementation language and
initial native backend required for the bootstrap; it is not ISK's design
template. The current compiler is an executable baseline, not a decision that
ISK source should look like Rust or that Rust ownership is ISK ownership.

## Status

All design choices below are **open**. The first compiler slice is not evidence
that a candidate is clearer, safer, faster, or cheaper for agents. A small
[controller pilot](../../benchmarks/results/2026-11-22-controller/README.md)
compares semantic versus text tooling within ISK, not language designs.
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
| Libraries and packages | Explicit or file-derived modules; qualified or unqualified imports; one version per workspace or isolated versions per dependency | Resolution stability, API lookup effort, conflict diagnosis, compatibility, and supply-chain policy compliance |

These approaches are hypotheses, not a ranking. Hybrid designs are possible.
Safety, human inspectability, and predictable diagnostics are hard constraints;
token count alone must never choose a design.
Also test module boundaries and dependency declarations for how well they
localize context, make change impact explicit, and support reproducible builds.
Explicit maps require upkeep but constrain discovery; file-derived modules
reduce declarations but require deterministic path and ambiguity rules.
One version per workspace simplifies identity but may block upgrades;
isolated versions may resolve more graphs but complicate type compatibility
and caching. These are open choices, not resolver requirements. See the
[package design](../packages.md) for staged trust and verification gates.

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

### Numerical pilot: greatest common divisor

See the [benchmark protocol](../../benchmarks/README.md) for the GCD
implementation and modification tasks, reference runner, agent-trial
requirements, result format, and history. Passing the reference checks is
not evidence of cheaper or more reliable agent authoring.

## Current evidence

The bootstrap integration tests exercise the CLI, including a factorial
program and arithmetic failures. The pilot now adds repeatable reference
runs across three languages. Neither establishes a language-design or
performance advantage. The controller pilot separately records agent-authoring
results, including all first submissions and infrastructure failures.

The single-file rename prototype has a correctness comparison: on an input
where a recursive function, a local binding, and a parameter share a name,
plain text replacement changes all three. The semantic edit changes only the
declaration and calls that resolve to it. This comparison measures edit
locality, not agent effort.

### Targeted symbol inspection (provisional)

Hypothesis: a checked signature and resolved call sites reduce the effort of
understanding a function before a single-file edit. Compare ISK `inspect`
against full-file reads and text search on the same tasks, including
recursion, nested calls, and same-name local bindings. The pilot target is at
least 20% fewer median context tokens among verified successes, with no
observed drop in final correctness; also report success rates, time, and
correction iterations across all runs. Inspection must be read-only, return
all and only resolved call sites in deterministic order, and not invoke the
backend. The interface does not change the compiled program or its runtime
behavior. For the impact report, `inspect --callers` exposes enclosing
functions directly; the text baseline must determine them from source.
The controller pilot bundles inspection with rename and records total tokens;
it does not measure inspection alone or the context-token target.

### Complete agent workflow experiment

Give an agent a small working ISK project and ask it to rename a recursive
helper, preserve behavior and unrelated names, verify the result, and report
which callers and tests were affected. Use equivalent variants with different
names and call shapes to limit memorization. The evaluator checks the rename,
preservation of unrelated source, runtime behavior, and accuracy of the
impact report. Separately test that conflicting, invalid, and stale plans
cannot modify the file.

Compare fresh independent sessions on the same ISK revision: both receive
the same task, test commands, model, tool/time budget, and access to normal
text tooling: grep or equivalent search, targeted line-range reads, ordinary
editing, and normal build and test commands. Use a larger, previously unused
fixture with multiple direct callers. Only one group also receives semantic
queries and validated edits. Counterbalance variant and group order. Run at
least five trials per group for this exploratory comparison and retain
failures. Record first and final correctness, context and total input/output tokens, correction
iterations, tool calls, time, and cost. Report success rates over all trials
and the median and spread among verified successes. Treat a 20% reduction in
median context tokens with no observed drop in final correctness as the pilot
target; use time and corrections to check whether the savings shifted effort
elsewhere. Do not infer a language-wide effect from a rename task.

Publish per-trial distributions, including which tools were actually used.
Separate input and output tokens, including tool descriptions in the total
cost of the assigned workflow. Report verification commands and time spent
verifying; do not substitute total tokens for missing context-only measures.
Repeat with a second model before generalizing beyond the first model.

Keep this comparison separate from the GCD language comparison: it tests
semantic tools while holding language fixed. The first pilot uses a
single-caller controller fixture; the richer workflow above remains unmeasured.

The current single-file reference workflow exercises `inspect --callers`,
`rename --plan`, `rename --apply`, `check`, and execution. The fixture verifies
that recursion and unrelated names are preserved and stale or conflicting
requests leave the file unchanged. Use full verification for this prototype;
it does not infer which test targets are affected.

### Controller language-design experiment

Keep the checked-in [controller](../../showcase/README.md) unchanged as the
regression baseline. Numeric state and event IDs allow combinations that the
compiler cannot distinguish by type. Nested conditionals also make it harder
to tell whether a missing case is intentional. Test whether a general-purpose
mechanism makes those mistakes easier to detect, not whether a shorter spelling
looks better.

- **Baseline:** numeric IDs and ordinary conditionals, with named constants
  and equivalent lookup tools where available.
- **First candidate:** distinct algebraic types for states and events, with
  exhaustive matching. This can reject swapped values and omitted cases,
  but exhaustive matches alone cannot prove a transition is correct.
- **Alternatives:** a typed transition table may make coverage easier to
  inspect but needs a clear rule for guards and defaults. A contract can
  express the fault-latch invariant across different implementations, but
  its checking cost and guarantees need separate evaluation. A semantic
  graph can expose relationships without changing source syntax; use this
  as a tooling-only comparison. Do not add a state-machine keyword for
  this experiment.

**First experiment:** use a small disposable prototype and the same
transition oracle for each variant. Preserve behavior for valid and invalid
inputs at the boundary, including unknown event IDs and the latched fault.
Ask fresh sessions to explain a guarded transition, then add one event
without changing existing transitions. Seed swapped state/event values,
an omitted branch, and a wrong but type-correct transition. Match the tool
conditions and report which defects the compiler, tooling, and tests catch.

**Decision criteria:** the typed candidate must reject the seeded type
confusion and missing cases it claims to prevent, and pass the same behavior
oracle. Measure comprehension accuracy, first and final correctness, input
and output tokens, corrections, and time to verified change. Measure native
runtime separately on the same workload; fix an acceptable regression margin
before running trials. Prefer the simplest candidate that improves the
verified workflow without reducing correctness or inspectability. If tooling
alone achieves the benefit, do not attribute it to the language.

### Attributing results

- **Language design:** compare equivalent tasks across ISK, Rust, and Python
  with comparable tool access. Also vary one design factor inside ISK to
  avoid attributing every difference to the language.
- **Semantic tooling:** hold language and task fixed; vary access to semantic
  operations. Where feasible, repeat with comparable tools in another language.
- **Combined environment:** evaluate the whole workflow on a reference
  application. Record host-provided capabilities and tooling differences
  explicitly. A combined win is not evidence that any one component caused it.

The main risks are mistaking the Rust lowering for language semantics,
expanding adapters before experiments identify a need, and generalizing from
small tasks or weaker baselines. Keep candidates provisional, preserve the
working bootstrap, and require evidence specific to the decision at hand.

### Dependency workflow experiment (proposed)

Start with a frozen local package catalog and a controller task: find a
compatible guard library, identify its public contract and required
capabilities, integrate it, and verify controller behavior. Compare source
and documentation lookup with a checked interface query on the same ISK
revision. Once versioning exists, extend the task to a safe upgrade and a
version conflict; use fixed catalog snapshots with competing origins and
known incompatibilities. Compare equivalent Rust and Python workflows
separately under matched requirements, fixtures, and agent budgets.

Pilot target: at least 20% lower median time to a verified integration among
successful runs, with no observed reduction in success rate or increase in
policy violations. Record tokens, tool calls, corrections, completion time,
verification results, and policy failures such as selecting an unexpected
source or treating an unverified claim as evidence. Report all failures
and variability. Keep evaluation offline on controlled fixtures; passing
the trial does not demonstrate general supply-chain security.
