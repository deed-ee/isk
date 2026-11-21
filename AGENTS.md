# ISK contributor instructions

**Build ISK for agents, not merely with agents.** This is the permanent
project mission. When choosing a design, ask what would work best if
programming languages were invented for autonomous coding agents today.
Design around how agents understand, construct, transform, and verify
software, not around existing languages or text-first workflows. Treat
human-readable source as one view of a semantic program. Let agents query and
edit checked semantics instead of reconstructing them from text.
Balance the five permanent [engineering pillars](docs/architecture.md):
agent-native language design, semantic-first representation, compiler-integrated
agent interfaces, empirical agent efficiency, and native performance, safety,
and correctness.

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

See [architecture](docs/architecture.md), [roadmap](ROADMAP.md), and the
[benchmark protocol](benchmarks/README.md).
