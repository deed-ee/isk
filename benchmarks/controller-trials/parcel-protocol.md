# Frozen follow-up: a realistic text baseline

Question: with targeted search and reads, does checked rename still reduce
total tokens **without fewer correct first submissions**? Exploratory signal:
at least 15% lower median total tokens using all allocated sessions, with no
missing token counts. This is a design target, not a significance test.

Use `run.py ISK NEW_BATCH --parcel` with the [same pinned binary and isolated
runner](README.md). The new parcel pricing file has four direct callers,
repeated calls, unrelated bindings and public executable tests. Both arms
receive identical task wording, a literal grep, line ranges, ordinary text
editing, `check`, `run` and `test`. Only the tool manifest differs: one arm
also has `inspect` and validated rename. Five independent sessions per arm,
interleaved with frozen seed, same model, budgets, environment and grader.
First/final correctness requires exact source bytes AND the distinct caller
list in declaration order. Exact bytes plus reference tests eliminate a need
to execute arbitrary submissions in the grading oracle.

Retain allocations, failures, input/output/total tokens, broker actions,
actual tool use, elapsed time, verification calls and corrections. Report
individual values, medians, ranges and uncertainty in correctness; do not
pool with the controller pilot. Repeated samples of one task and model do
not establish language effects. Fixture size and task difficulty also change:
this is not a causal estimate of the effect of adding grep alone.
