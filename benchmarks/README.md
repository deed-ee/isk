# Agent Development Efficiency benchmarks

The permanent target is less total agent effort per correct, verified,
maintainable software change, subject to safety, determinism, runtime, and
memory constraints. This pilot supports ISK, Rust, and Python without
requiring new ISK syntax.

## Public tasks

- **`gcd-u64` (implement):** Implement `gcd(a, b)` for two unsigned 64-bit
  integers. Return their greatest common divisor as an unsigned 64-bit
  integer; `gcd(0, 0) = 0` and `gcd(48, 18) = 6`. Do not call a predefined
  GCD function, use external packages, read input, or print. Submit the
  function only; the evaluator supplies the entry point.
- **`gcd-coprime` (modify):** In the provided working GCD implementation,
  add `coprime(a, b)` with the same input domain. Return a Boolean that is
  true exactly when `gcd(a, b) = 1`. Reuse `gcd`, preserve its behavior, and
  follow the same library and I/O restrictions. For example,
  `coprime(8, 15)` is true and `coprime(0, 0)` is false.

Add debugging, refactoring, cross-module changes, unfamiliar-project
comprehension, and application extensions as the compiler supports them.
Rotate tasks and hold out validation cases; don't tune the language to pass
this pilot.

The [controller task catalog](controller-tasks.md) defines the first
showcase-based bug fix, extension, and semantic refactor. These experiments
are separate from the existing numerical reference runner.

## Reproduce the reference baseline

```sh
python3 benchmarks/pilot.py --task all --trials 3 > pilot-results.jsonl
```

This runs only the checked-in implementations. Each task uses the same
vectors in all three languages; the modification task also reruns the GCD
checks. The runner requires Python 3 and the project's Rust toolchain.
`--trials 1` is the lightweight CI check. `wall_time_ms` is program build
time for reference records (null for Python); `runtime_ns` includes process
startup, computation, and output after one warm-up. Don't use these tiny
programs to rank language performance or agent productivity.

## Run and report agent trials

Use independent fresh sessions. Supply the matching public task, toolchain
documentation, and starter code (for the modification). Supply equivalent
capabilities and pin model/version, sampling settings, tools, time and
iteration budgets, harness revision, and toolchain for each comparison
group. Randomize trial order. Do not expose reference solutions (except the
modification starter) or validator fixtures in agent workspaces. Evaluate
submitted code in an isolated environment; this runner must not execute it.
Review maintainability under the same rubric in all groups.

Write one JSON Lines record per trial using [the result schema](result.schema.json).
Version 1 records from the reference runner contain no authoring metrics.
Version 2 adds agent authoring metadata and measurements:

- Count all input/output model tokens using the named tokenizer, plus tool
  calls, agent iterations, compilation failures, test failures, and correction
  iterations. Count iterations as model turns and corrections as revisions
  after the first submitted implementation. `context_tokens` is the portion
  of input tokens spent on repository or program context.
- Verification experiments should also record `validation_cycles` and
  `test_cycles` (distinct validation and test invocations, including failures)
  and `test_time_ms` (elapsed time spent executing tests, separate from
  program runtime). These optional version 2 fields preserve older records.
- Record `wall_time_ms` from task start to submission of the finally verified
  change (or trial stop for failures). Record final verification and whether
  the first submitted implementation passed the same requirements. Capture
  per-trial model/tool cost in USD if available, including failed attempts.
- Report success rate across independent runs and variability or uncertainty
  alongside timing and effort. Report cost per verified success as total cost
  across **all** trials divided by the number of verified successes; report
  unavailable if costs are missing or there are no successes. Leave missing
  measurements `null`, never zero or interpolated.

Keep batches of raw results, including failures, in `benchmarks/results/` or
link them from the benchmark history if stored externally. Name the batch by
date and suite, retain exact prompts and settings, and label reference-only
runs separately. Don't replace old batches when the compiler, model, or
tasks change.

For verification experiments, use seeded controller repairs and compare
focused-first verification against full-first verification on the same ISK
revision. Both groups must pass the same full final suite. Report failures
missed by the focused pass, end-to-end time and tokens, test execution time,
validation and correction cycles, success rate, and cost per verified change.
Run equivalent task groups in Rust and Python separately to distinguish
language and tooling effects. No focused-selection advantage has been measured.

## History

- [2026-11-21 reference baseline](results/2026-11-21-reference.jsonl):
  three repetitions per task and language, 18 records total. These are
  correctness and startup/build timing measurements, not authoring results.

No controlled agent-authoring results have been collected.

The [proposed dependency workflow experiment](../docs/research/design-evaluation.md#dependency-workflow-experiment-proposed)
will measure package discovery, integration, and update effort once local
interfaces exist. It does not change the numerical pilot.
