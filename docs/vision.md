# ISK — Agents for Agents

If programming languages were invented for autonomous coding agents today,
what would a program be, and how would an agent know it had changed the
right thing? ISK exists to investigate that question. Rust lets us ship an
executable baseline today; it does not answer it. Neither Rust compatibility,
transpilation nor a particular agent protocol defines success.

**Objective: minimum total computational effort per correctly implemented,
verified and maintainable software change, subject to strong safety and
native-performance constraints.** Count understanding, generation, review,
failed attempts, debugging, verification and future maintenance. More compact
syntax that saves output tokens but causes another repair cycle can be a loss.
Report success, effort and runtime constraints separately; do not collapse
unknowns into a favorable score.

### Design for the entire change

Prefer representations that make intent and ambiguity visible, prevent
mistakes, support static checks and let an agent retrieve just the context
it needs. Test how these properties survive more symbols, modules, authors
and revisions. For each major feature, describe how an agent **expresses,
discovers and modifies** it, how the compiler **verifies** it, how an agent
**diagnoses failure**, and what happens **at project scale**. Design the
language, compiler, semantic API, package interfaces and verification loop
together.

### Give programs an interface beyond text

Explore typed semantic representations, identities that survive edits,
dependency graphs, machine-readable contracts, structured construction,
transactional transforms and precise impact queries. An agent might submit
a checked graph edit rather than regenerate a file. Keep a human-readable,
reviewable projection: a contributor must still be able to inspect a diff,
use version control, interchange source and diagnose what the tools did.
Test round trips and unchanged regions. A graph is a candidate, not a
replacement mandate; an MCP adapter is a client, not the semantic model.

### Preserve application quality

Investigate memory safety, strong typing, explicit effects and error handling,
predictable resource use, efficient concurrency, native execution and
reproducible builds. Benchmark the produced application as well as its
authoring workflow. A property inherited from one backend is not yet an
independent ISK guarantee. See the [capability matrix](capabilities.md)
before making claims, and the [innovation gate](architecture.md#language-innovation-gate)
before adopting a language mechanism.

### Architectural risks to revisit

| Risk | Early warning | Response |
|---|---|---|
| Bootstrap lock-in | Rust emission is the only definition of a feature | Specify observable semantics and negative cases before lowering. |
| Tooling displaces language research | Milestones only add commands or protocol endpoints | Pair tooling milestones with a competing language representation and a tooling-only control. |
| Two conflicting programs | Graph edits and source diffs disagree | One checked snapshot, revalidation, source-preservation tests and stale-edit rejection. |
| Benchmark overfitting | Gains only on known symbol names or against whole-file reads | New fixtures, strong text baselines, unseen tasks and independent replication. |
| False verification confidence | Fast selected tests or dependency declarations presented as guarantees | Compare against full regressions; distinguish declared, verified and enforced properties. |
| Complexity or runtime cost migrates downstream | Fewer prompt tokens but more failures, slower binaries or harder reviews | Measure total effort, failure distribution, native runtime, memory and human review separately. |
