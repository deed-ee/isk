# Capability matrix

**Implemented** = working bootstrap behavior with executable coverage.
**Experimental** = runnable prototype with a bounded contract.
**Designed** = documented interface or architecture, no working capability.
**Proposed** = candidate awaiting evaluation. **Unexplored** = no concrete
design or evidence here. These labels apply to the specific slice, not to
every future version of the language.

| Area | Works today | Beyond today's boundary |
|---|---|---|
| Constructs and types | Implemented: single-file functions, calls, local bindings, `u64`, `bool`, unit, checked arithmetic, comparisons and `if`. | Proposed: nominal domain types, inference alternatives, exhaustive matching and typed failures. No final syntax. |
| Projects and modules | Implemented: `new`, `check DIR`, `run DIR` resolve the scaffold's `src/main.isk`; explicit-file commands remain available. | Designed: [project loader evolution](projects.md). No manifest required or interpreted, imports, multi-file visibility or workspace resolution. |
| Libraries and packages | No ISK package manager; Cargo dependencies implement the bootstrap. | Designed: [local-first package stages](packages.md). Registry, lockfile resolution and update intelligence are not available. |
| Semantic API | Experimental: checked snapshot, typed signatures, snapshot-local IDs, resolved calls, byte ranges, CLI JSON. | Designed: protocol-independent API consumers and read-only MCP adapter. Proposed: durable IDs and editable typed representation. |
| Structured transformations | Experimental: checked single-function rename, plan, stale-snapshot rejection and atomic file replacement. | Proposed: structured construction, cross-file transactions and impact queries. No concurrent-writer guarantee. |
| Contracts, effects, capabilities | No general language constructs or enforcement for these. | Proposed: inspectable invariants/effects. Unexplored: a verified contract proof format for third-party packages. |
| Testing and diagnostics | Implemented: CLI source diagnostics and full bootstrap regressions. Experimental: explicit boolean test roots, per-symbol results and advisory `affected-tests`. | Proposed: incremental verification and property checks. No test discovery or coverage guarantee. |
| Runtime and native compilation | Implemented: `build` and `run` emit Rust and invoke `rustc` to produce a native executable. `check` needs no backend. Overflow/division errors fail at runtime. | Proposed: independent performance/resource targets, concurrency and backend evaluation. No stable ABI, portable resource bound or bit-reproducibility claim. |
| Security and memory safety | Bootstrap restricts inputs by its type/grammar subset; project roots reject escaping entry paths. | Proposed: capability enforcement and provenance verification. Native execution is not sandboxed by the ISK CLI; the experiment runner's isolation is not a language guarantee. |

Evidence: [CLI contract](cli.md), [end-to-end tests](../tests/e2e.rs),
[ADR for explicit tests](adr/0002-explicit-test-runner.md) and
[ADR for project roots](adr/0003-project-root.md).
