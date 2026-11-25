# Controller maintenance benchmark tasks

These task specifications use the [Stage A controller](../showcase/README.md#controller).
They are not yet part of the numerical pilot or ordinary CI. Freeze the
starter revision and preserve the numeric state/event IDs and public behavior
unless the task explicitly changes them.

| Task | Starting point and request | Acceptance |
| --- | --- | --- |
| `controller-ready-guard` | Seed a defect that ignores the interlock on Ready; diagnose and fix it. | Starting stays Starting if the interlock is blocked, advances if clear, and all other transitions are unchanged. |
| `controller-timeout` | Add a `StartTimeout` event with ID `5`. | It trips only Starting into Fault, regardless of the interlock. Existing events, IDs, and the fault latch remain unchanged. |
| `controller-rename` | Rename the `advance` helper to `advance_without_fault` and report its direct callers. | The declaration and resolved calls change, the transition contract and demo output do not, and the caller report is accurate. |

Prepare equivalent ISK, Rust, and Python starters for the behavioral tasks:
same transition contract, same seeded defect where applicable, same public
examples, and equivalent withheld tests of state/event combinations,
interlocks, and multi-event traces. Supply only the assigned starter
and public requirements to agents. Run independent sessions with model,
tools, and limits pinned; retain failures and capture the metrics in the
[benchmark protocol](README.md#run-and-report-agent-trials).

For the rename, compare ISK text tools alone against the same ISK revision
with semantic inspection and validated edits available. Check unrelated names
and reported impact as well as behavior. Hold language fixed so this measures
tooling, not syntax. See the [rename and repair pilot](results/2026-11-22-controller/README.md)
for the first within-ISK results; cross-language trials remain open.

### Reproduce the controller rename task

From a full clone, run `python3 benchmarks/controller-rename/evaluate.py
prepare /path/to/new-directory`. Both groups receive **only** the resulting
`main.isk` and `task.md`, outside the evaluator checkout. The starter comes
from commit `119eaa8aaeab98758dd7086f6003d1311b9252b1`, with one local
renamed to match the helper. Supply the same pinned ISK CLI and common
`check`/`run` and text tools; allow `inspect` and `rename` only for the
semantic group. Keep evaluator files and Git history out of both workspaces.

Collect the two submitted files, then run `python3
benchmarks/controller-rename/evaluate.py grade /path/to/submission`. The
evaluator checks exact source bytes and the caller report independently,
prints both verdicts as JSON, and exits zero only when both pass. It does
not run submissions or use the compiler's rename implementation as its
answer key. Run `python3 benchmarks/controller-rename/reference.py` to
exercise the CLI reference edit, compare its demo trace, and check the
blind-replacement and incorrect-caller controls. This is an evaluator
smoke test, not a model trial. For independent sessions and measurements,
follow the [agent trial protocol](README.md#run-and-report-agent-trials).
