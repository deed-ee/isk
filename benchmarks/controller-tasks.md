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
tooling, not syntax. No agent results have been collected for these tasks.
