# What should agents program?

**Research agenda, not a language specification.** The unit of success is a
verified, reviewable maintenance change. These are proposed gates, not
measured results. Freeze tasks, hidden cases, budgets and thresholds before
trials. Retain failures. Do not save effort by skipping verification.

## Questions and first discriminating experiments

| Problem | Alternatives and trade-offs | Hypothesis, experiment and success criterion |
|---|---|---|
| **Representation:** agents reconstruct relationships from text. | Text is diffable; ASTs remove parse ambiguity but can be verbose; typed nodes encode identity but can go stale. A hybrid needs a lossless projection. | Construct the same helper and calls from text, a tree edit or a typed transaction on unseen layouts. Require equivalent output, zero unrelated byte changes and stale-edit rejection. Seek ≥20% fewer median corrections before preferring the more complex representation. |
| **Types:** states and events share a numeric representation, so swapped arguments type-check. | Named types prevent confusion but need conversions; inference reduces annotations but can move errors away from their cause; linting is cheaper but optional. | Seed state/event swaps at call boundaries. Compare wrappers plus lint with distinct checked types at similar verbosity. Require rejection of every swap, acceptance of valid controls, no extra allocation and fewer failed repairs. |
| **Control flow:** implicit priority in nested branches hides omitted cases. | Ordered branches express precedence; exhaustive matching exposes holes; tables are compact but defaults may hide overlap. | Add an event to a held-out controller variant. With the same transition vocabulary, measure detection before execution. Require no missed seeded omissions, no false alarms and unchanged legacy traces. |
| **Errors:** unit returns or panics can obscure expected failures. | Typed results encourage explicit recovery but add propagation code; checked failure effects separate propagation from recovery; exceptions shorten the normal path but hide exits. | Add a fallible local decode and propagate a recoverable error across two calls. Require all seeded unhandled paths diagnosed and no lost recoverable failure; compare correction cycles and happy-path cost. |
| **Contracts:** local types do not express a cross-step invariant. | Boolean tests check examples; pre/postconditions describe boundaries; inductive invariants can be expensive or undecidable. | Modify fault-reset behavior with tests alone versus an explicit invariant. Mutation-test both: require detection of every seeded latch violation and no rejected valid trace; report verifier time and author effort. |
| **Effects and capabilities:** signatures may hide transitive I/O. | Comments are unchecked; inferred effects reduce writing but can broaden silently; explicit capability parameters are visible but add plumbing. | Insert logging below a pure helper. Compare docs, inferred summaries and checked capabilities. Require every transitive violation caught before run and no false positives on pure controls; measure context and API churn. Distinguish declaration from enforcement. |
| **Concurrency:** ordering, cancellation and cleanup errors resist reproduction. | Structured scopes constrain lifetimes; actors isolate state but need backpressure semantics; locks allow control but expose ordering hazards. | Model a bounded two-worker queue with cancellation in host prototypes first. Require no lost/duplicate completions and bounded memory under deterministic schedules; measure throughput, peak memory and repairs before choosing an ISK model. |
| **Modules and packages:** finding a public interface requires source knowledge. | File-derived modules save metadata but can make moves semantic; explicit exports and dependencies improve queries but need upkeep. | Extract a local controller library. Compare reading files, an API index over files, and checked public/private boundaries. Require repeatable clean builds, all private-access negatives rejected and successful integration without reading private bodies. |
| **Structured editing:** a plausible patch can silently change another binding. | Text edits are general; AST patches preserve syntax but not binding; symbol transactions validate references but need identity and rollback rules. | Compare a parameter rename across scopes with grep/text, structural substitution and a checked operation. Require zero unintended edits, retained formatting and no writes on stale/conflicting edits. Measure first-pass correctness and total tokens. |

Where correctness saturates or the baseline median is zero, report a ceiling
or floor, not a percentage improvement. Include review effort and negative
controls; shorter examples do not select a design.

## State-machine case: keep behavior fixed

Keep the [numeric controller](../../showcase/controller/src/main.isk) as an
unchanged executable reference: numeric state/event IDs, nested conditionals,
a boolean interlock and a pure transition function. Preserve invalid-ID
handling at the boundary.

| Candidate | What changes? | What could go wrong? |
|---|---|---|
| Conventional conditionals | Existing semantics; improve layout only | Clearer spelling still misses guarded transitions. |
| Algebraic types plus matching | Distinct domains and exhaustiveness | Catch-all patterns erase exhaustiveness; external IDs need decoding. |
| Data-driven table | Transitions become inspectable data | Duplicate keys, rule order and fallback behavior. |
| Declarative transitions | Reject conflicting or uncovered rules | Domain specialization and poor reuse outside controllers. |
| General contracts | Retain functions; state invariants separately | Weak contracts pass broken transitions; checking cost. |
| Typed graph | Query guards, transitions and identities directly | Lossy round trips or unreadable diffs. |

Begin with paper/host-language prototypes, not a state-machine keyword.
Freeze an independent oracle: every known state × event × interlock,
unknown IDs and multi-step traces including blocked Ready and latched Fault.
Ask fresh agents to implement, predict unseen traces, add an event and
diagnose invalid/ambiguous transitions. Count implementation tokens,
predictions correct, first/final modification correctness, invalid transitions
detected, verification calls/time and repairs. Measure optimized native
runtime and memory separately on a repeated workload with a checked checksum,
fixed hardware/profile and warm-ups. Mark non-native prototypes separately.

**Separate spelling from meaning:** render identical checked transitions in
two surface forms; then fix the surface form and vary the type/exhaustiveness
checks or semantic queries. Include a lint/editor alternative over ordinary
functions. Prefer the least complex general abstraction that survives held-out tasks.

## Attribution: three distinct comparisons

1. **Language design.** Within ISK, change one rule while fixing tooling and
   observable behavior. Separately run equivalent implementation, comprehension,
   debugging, modification and testing tasks in ISK, Rust and Python with
   the same contract, fixtures, realistic search/edit tools, budgets and
   equivalent validation commands. Report author effort, correctness and
   program runtime separately. Model familiarity, interpreter startup and
   compiler maturity are confounders, not intrinsic language effects.
2. **Tooling.** Within each language compare strong text tools against the
   same tools plus semantic operations. Try rust-analyzer and a Python
   language service on resolvable symbols. Pretest shadows, dynamic calls
   and stale edits; publish a capability-parity table and mark unsupported
   cells. Within-ISK pilots estimate access effects, not language effects.
3. **Combined environment.** Compare full stacks on one change crossing a
   public interface, dependency and test boundary once these work in ISK.
   Allow normal package and verification tools for each stack. Report
   first/final success, effort per success (including failures), runtime,
   memory and reviewability. Contrast with text-only and within-language
   cells, identifying interactions and missing cells. A full-stack win
   cannot by itself select a type system or justify a protocol.

Randomize variants and arm order within pinned model/toolchain blocks.
Reserve unseen fixtures. Publish raw per-session data and distributions,
not a pooled headline across unlike tasks. See the [protocol](../../benchmarks/README.md).
