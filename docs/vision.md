# ISK vision: Agents for Agents

**Build ISK for agents, not merely with agents.** ISK is an independent
programming language and development environment. Its research question is:
what would programming look like if autonomous agents were a primary audience
from the start?

The objective is to minimize total effort per correctly implemented, verified,
and maintainable change: understanding, construction, editing, diagnosis,
correction, and verification. Count computation, time, and tool use across the
whole workflow. Shorter syntax alone is not a win.

- **Design the language for agent work.** Prefer semantics agents can understand,
  generate, and modify reliably. Evaluate static guarantees, diagnostics, and
  maintenance at project scale, not just authoring a small example.
- **Make checked semantics a first-class interface.** Explore typed program
  representations, durable identities, dependency graphs, validated
  transactions, and precise context and impact queries. Text may be one view
  rather than the only editing interface. Preserve human readability, review,
  version control, and interoperability.
- **Co-design language and tools.** The compiler, semantic API, package model,
  and verification system must reinforce the language. CLI and optional MCP
  expose checked operations; neither defines the language or its priorities.
  Rust is a practical bootstrap and backend, not a dialect to inherit or a
  constraint on ISK's eventual semantics.
- **Preserve application quality.** Evaluate native performance, safety,
  determinism, and resource usage separately from agent effort. Treat strong
  typing, explicit effects, concurrency, and reproducible builds as research
  goals where not already demonstrated. Do not promise unverified guarantees.

Keep the [engineering pillars and decision gates](architecture.md) connected.
Compare language choices, tooling, and the combined environment separately
before attributing an improvement. The [capability matrix](capabilities.md)
describes what works today; the [roadmap](../ROADMAP.md) describes what to test
next.
