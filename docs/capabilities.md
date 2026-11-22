# ISK capability matrix

This describes the bootstrap, not promises for a release. **Implemented**
means working within the documented scope. **Experimental** means runnable but
provisional. **Designed** means a staged design exists, not an implementation.
**Proposed** means an experiment or candidate is defined but not selected.
**Unexplored** means no design has been selected or evaluated here.

| Area | Status | Scope and limits |
| --- | --- | --- |
| Language and types | Implemented | Single-file functions, local bindings, branches, `u64` and `bool`; checked calls and expressions. The current syntax and type system are provisional. |
| Application roots | Implemented | `new` scaffolds `src/main.isk`; `check` and `run` accept an explicit directory with that entry. No manifest, module discovery, or workspace resolution. |
| Modules and packages | Designed | [Staged local modules, interfaces, and dependency resolution](packages.md); no ISK package manager or registry. |
| Semantic API | Experimental | Checked snapshots, typed signatures, and resolved call sites. Function IDs are snapshot-local, not durable across declaration changes. No persistent workspace service. |
| Structured editing | Experimental | Checked single-file rename preview, plan, and apply; stale plans are rejected. No general transactional editing API. |
| Verification | Experimental | Explicit named boolean roots and symbol-linked reports; advisory affected-root selection. No test discovery or automatic focused verification. |
| Contracts and effects | Proposed | [Research candidates](research/design-evaluation.md), not user-facing guarantees or enforced capabilities. |
| Runtime and native compilation | Implemented | `build` and `run` lower to Rust and invoke `rustc`; `check` does not. Integer overflow and division by zero fail at runtime. This is not evidence of performance parity with other languages. |
| Network applications | Proposed | The [offline controller](../showcase/README.md) works today. A host-backed REST prototype is planned; ISK does not provide HTTP, JSON, or MQTT. |
| Security and concurrency | Unexplored | No verified execution sandbox or ISK concurrency model. The package trust design does not enforce permissions or provide supply-chain guarantees. |
