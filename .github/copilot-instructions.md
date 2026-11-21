# ISK instructions for Copilot

Follow [AGENTS.md](../AGENTS.md). ISK is designed for agents, not merely usable
by agents. Build around checked semantic queries and validated edits, with
human-readable source as one view. The semantic API must not depend on CLI,
MCP, or Rust code generation. Preserve the working bootstrap without treating
its subset or Rust conventions as long-term language constraints.
Use the same checked operations for all interfaces. MCP is an optional
adapter: expose only implemented operations and verify parity with the CLI.

Use lightweight local checks for routine work. For major changes, document
the expected benefit, baseline, success criteria, and evidence in the relevant
ADR. Measure total effort to a verified change, preserve safety, determinism
and native performance, and distinguish tooling effects from language effects.
Do not claim an agent-efficiency improvement from reference-only results.
Optimize the full development and verification loop. Use focused tests for
early feedback only; fall back to broader tests on uncertainty and keep
full regression testing in CI.

Keep modules and package intelligence in the shared semantic API, independent
of Rust. Prefer explicit local dependencies first. Treat package metadata as
untrusted data, distinguish declared permissions from enforcement, and never
claim supply-chain guarantees the implementation cannot verify. See the
[staged package design](../docs/packages.md).
