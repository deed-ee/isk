# ISK contributor instructions

**Build ISK for agents, not merely with agents.** This is the permanent
project mission. When choosing a design, ask what would work best if
programming languages were invented for autonomous coding agents today.
Design around how agents understand, construct, transform, and verify
software, not around existing languages or text-first workflows. Treat
human-readable source as one view of a semantic program. Let agents query and
edit checked semantics instead of reconstructing them from text.
Balance the eight permanent [engineering pillars](docs/architecture.md),
including a shared semantic API, agent interfaces, and efficient verification.

- For significant language, compiler, tooling, or roadmap changes, state the
  agent benefit as a hypothesis. Record the baseline, measurable success
  criteria, correctness and runtime constraints, results, and limitations in
  the relevant ADR. Keep unmeasured designs provisional.
- Use the smallest useful implementation. Reuse the compiler's checked
  semantic information for agent tools, keep output machine-readable where
  practical, and validate edits before returning or writing them. Keep the
  semantic API independent of CLI, MCP, and code generation. Preserve the
  working bootstrap without making its Rust backend or subset design defaults.
- Run the lightweight checks in [README.md](README.md) for ordinary changes.
  Run controlled agent comparisons at meaningful milestones, not on every
  change. Compare semantic versus textual workflows within ISK separately
  from cross-language comparisons. Never confuse reference runs with agent
  runs or omit failed trials.
- Optimize the full verification loop, including testing, diagnosis, and
  correction. Treat focused tests as advisory, fall back to broader tests
  when impact is uncertain, and keep full regression testing in CI.
- Keep small language demonstrations in `examples/`, controlled experiments
  in `benchmarks/`, and growing reference applications in `showcase/`. Use
  real maintenance problems from the showcase to evaluate language changes;
  don't add features just to make a fixture pass.
- Treat [library and dependency intelligence](docs/packages.md) as part of
  the semantic API. Prefer explicit local dependencies first. Distinguish
  declared capabilities and provenance claims from verified evidence and
  actual enforcement; do not treat backend dependencies as trusted ISK packages.
- Keep [application projects](docs/projects.md) usable without agent prompt
  files. Derive semantic project information from checked source, not
  manually maintained parallel metadata.

See [architecture](docs/architecture.md), [roadmap](ROADMAP.md), and the
[benchmark protocol](benchmarks/README.md).
