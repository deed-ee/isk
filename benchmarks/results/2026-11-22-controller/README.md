# Controller agent pilot — 2026-11-22

Two small maintenance tasks, five fresh model sessions per arm per task.
See the [runner and protocol](../../controller-trials/README.md) for the
tool sets, budgets, grading rules, and reproducibility limits. The unchanged
compiler was built at `6b68ec828713ab3072ee34ba1624f64270fc4f98`.

## Model results

`openai/gpt-6.2-astra`, OpenCode v2.3.2, runner `ec3aa19`. All 20 first
submissions passed; no corrections. Median and full observed range:

| Task | Arm | First correct | Total tokens | Seconds | Used semantic ops |
|---|---|---:|---:|---:|---:|
| Rename | Text | 5/5 | 2012 (2011–2053) | 10.0 (6.7–10.8) | — |
| Rename | Semantic | 5/5 | 1544 (1512–1567) | 8.0 (7.6–11.5) | 5/5 |
| Repair | Text | 5/5 | 1988 (1950–2008) | 8.1 (6.7–10.5) | — |
| Repair | Semantic | 5/5 | 2068 (2040–2072) | 10.0 (8.2–12.8) | 0/5 |

**Interpretation:** access to semantic tools coincided with about 23% fewer
median total tokens on the targeted rename. The text broker offered literal
edits and full-file reads but no targeted text search: a stronger baseline
could narrow this gap. The held-out repair showed no similar advantage;
all ten edits restored the original bytes. Merely describing extra tools
also adds tokens. The tasks did not separate the groups on correctness.
With five sessions per cell, one model, overlapping timing ranges and no
context-only token or cost attribution, this does not establish a general
speedup or meet a *context-token* evidence gate. Replicate on a larger
unfamiliar fixture with grep/range reads, more than one direct caller and
a second model before expanding the API on the strength of this result.

**Check the evidence:** [per-trial measurements and exact task prompts](results.jsonl),
[turn-by-turn actions, replies and submitted files](evidence.tar.gz), and
[compiler binary digest](binary.sha256). From the repo root:

```sh
data=benchmarks/results/2026-11-22-controller
work=$(mktemp -d)
cp "$data/results.jsonl" "$work/"
tar -xzf "$data/evidence.tar.gz" -C "$work"
python3 benchmarks/controller-trials/verify_batch.py "$work"
python3 benchmarks/controller-trials/summarize.py "$work"
```

## Infrastructure history

The first sandbox preflight failed to find the distribution's `cc` symlink
target. No participant ran; binding the system linker alternatives fixed it.
Then [20 startup records](startup-failures.jsonl) were produced before
OpenCode could load the participant profile. The CLI inherited a `PWD` that
did not match its working directory. There were **zero model submissions**;
these are infrastructure failures, not incorrect model answers. No trial
was replaced selectively. The entire schedule was rerun after setting `PWD`
to the fresh client directory. The primary model comparison uses that second
batch. Do not pool the startup failures with it.
