# Within-ISK controller pilot

This experiment compares **access** to semantic operations with a text-only
workflow on the same compiler. It does not compare ISK with another language.
Primary endpoint: a correct first submission, including the caller report
for the rename. Secondary: final success, elapsed time, token counts, semantic
operations actually used, and corrections. No timing or token target was
chosen after observing outcomes.

The frozen schedule runs five fresh sessions per arm on the rename, then
five per arm on a previously withheld Ready/interlock repair. Arm order is
shuffled within task (seed `221126`); sessions run sequentially. Both arms
use one broker, text operations, `check` and `run`; the semantic arm adds
`inspect` and validated `rename`. The participant is not told the exact
answer, prior sessions, or other participants' scores. Each invocation uses
the model and budgets in `run.py`, with a maximum of two submissions.

`submit` copies the actual submitted files **before** grading or feedback.
Rename uses the independent byte/caller oracle. Repair compares the demo
and a generated transition driver against the original unseeded program:
72 state/event/interlock combinations and one three-step trace. Before
starting, the runner checks that the reference executes and that the seeded
defect fails the matrix. This behavioral oracle is for small in-place edits;
its driver assumes helpers precede `fn main()`. It is not a general
equivalence checker.

## Reproduce

Linux host with user namespaces, bubblewrap, util-linux, `/usr` toolchain
layout, Python 3.10+, Rust, Git history, and authenticated OpenCode V2
access to the exact configured model:

```sh
base=$(mktemp -d)
git worktree add --detach "$base/compiler" 6b68ec828713ab3072ee34ba1624f64270fc4f98
cargo build --locked --manifest-path "$base/compiler/Cargo.toml" --target-dir "$base/target"
python3 benchmarks/controller-trials/run.py "$base/target/debug/isk" "$base/batch"
python3 benchmarks/controller-trials/verify_batch.py "$base/batch"
python3 benchmarks/controller-trials/summarize.py "$base/batch"
git worktree remove "$base/compiler"
```

Use a fresh checkout of the recorded **runner** revision and do not switch
branches while it is running. Keep the binary digest and results with the
batch. These are live model calls, not part of CI. Use a separate model
session for each trial; the runner continues only within a trial. OpenCode's
participant profile denies all native tools. A host-side broker implements
a fixed action vocabulary and fixed file operands. ISK commands (including
evaluation of submitted repair programs) run inside bubblewrap without the
host home, network, checkout, or model credentials. The compiler, system
tools and a writable per-invocation /tmp are available.

`results.jsonl` holds one v2 record per assigned trial; `turns.jsonl` and
numbered submission files preserve what the model returned and what was
graded. Multiple JSON actions may share a turn. The broker counts actions,
not CLI processes. A correction is a **second submission**. Elapsed time
includes model, CLI and grading latency; neither provider sampling seeds
nor cache state are controlled. Token totals use OpenCode step-finish
events (input plus cache reads/writes; output plus reasoning). No independent
context-only token or billing figures are available. `failed_tests` and
`failed_builds` count failing `run` and `check` requests respectively;
they do not classify backend errors within `run`.

Small samples from a single model, tiny fixtures and a constrained editing
interface make these **method pilots**. Compare within task; a mixed or null
result does not establish general language productivity.
