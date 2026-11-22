# Multi-caller rename with targeted text search

Ten fresh model sessions on a newly written 93-line parcel pricing file
(versus the original 76-line controller). Four distinct direct callers, five
resolved calls and unrelated same-name bindings. [Protocol and threshold](../../controller-trials/parcel-protocol.md)
were frozen at `c613768` before trials. Same `openai/gpt-6.2-astra` model,
budgets and pinned ISK binary as the previous pilot. All four public tests
passed on the reference fixture before the first session.

| Arm | First / final correct | Median total tokens (min–max) | Median seconds (min–max) | Median broker actions |
|---|---|---|---|---:|
| Text + targeted search | 5/5 · 5/5 | 2902 (2310–3367) | 15.8 (10.4–25.9) | 11 |
| Same tools + semantic operations | 5/5 · 5/5 | 1710 (1666–1735) | 9.7 (7.5–18.3) | 6 |

**What actually happened:** four text sessions used grep, two used range
reads; all five semantic sessions used both inspection and rename. Every
session invoked the test command once; four per arm also requested a source
check. No corrections, failed checks or failed test runs. Median total tokens
were 41% lower in the semantic arm, meeting the prespecified exploratory
target. The broker's checked rename replaces multiple ordinary text edits.
That is useful evidence about this operation, not about the language syntax.

| Arm, trial | Input | Output | Total | Seconds |
|---|---:|---:|---:|---:|
| Text 1 | 3122 | 245 | 3367 | 16.0 |
| Text 2 | 2690 | 212 | 2902 | 15.8 |
| Text 3 | 2114 | 196 | 2310 | 10.4 |
| Text 4 | 2716 | 219 | 2935 | 25.9 |
| Text 5 | 2262 | 214 | 2476 | 11.0 |
| Semantic 1 | 1649 | 86 | 1735 | 9.7 |
| Semantic 2 | 1622 | 92 | 1714 | 18.3 |
| Semantic 3 | 1619 | 61 | 1680 | 10.0 |
| Semantic 4 | 1619 | 91 | 1710 | 8.9 |
| Semantic 5 | 1602 | 64 | 1666 | 7.5 |

**Uncertainty:** even treating sessions as independent Bernoulli draws,
5/5 gives a 95% Wilson interval of roughly 57–100% per arm; equal observed
scores do not establish equivalence. Task and model were *not* sampled from
a population. Times include provider latency and overlap. Both arms ran
through a constrained single-file broker (literal grep, not a full shell or
IDE); token counts include repeated model context, not only source. The file
and number of callers differ from pilot 1: do not compare percentages as a
causal effect of grep. There is no Rust/Python or language-semantics arm.

[Per-trial JSONL](results.jsonl) contains the exact prompts, budget,
verification counts, corrections, tool access and measurements.
[Turn logs and frozen submissions](evidence.tar.gz) permit independent
inspection. Reproduce or verify from the repo root:

```sh
data=benchmarks/results/2026-11-22-parcel
work=$(mktemp -d)
cp "$data/results.jsonl" "$work/"
tar -xzf "$data/evidence.tar.gz" -C "$work"
python3 benchmarks/controller-trials/verify_batch.py "$work" --parcel
python3 benchmarks/controller-trials/summarize.py "$work"
```
