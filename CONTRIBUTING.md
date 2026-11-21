# Contributing to ISK

Welcome! ISK is an early, research-driven programming language and development
environment designed for AI coding agents. We're testing whether designing for
agents from first principles can make software development more efficient,
correct, and reliable. The bootstrap is not the final language, and constructive
criticism, counterexamples, and negative results are welcome.

You don't have to submit code. Useful contributions include reproducible bug
reports, examples, documentation, benchmark tasks and methodology, and design
feedback. Code contributions can focus on the compiler and parser, language
semantics and type safety, semantic representation and agent tooling, testing
and verification, or the foundations of modules and packages.

## Get started

Install Git, [Rust via rustup](https://rustup.rs/), and a native linker for your
platform. The [toolchain file](rust-toolchain.toml) selects stable Rust, rustfmt,
and Clippy. Python 3 is needed only for the benchmark smoke check.

```sh
git clone https://github.com/deed-ee/isk.git
cd isk
cargo build --locked --bin isk
cargo run --locked -- check examples/factorial.isk
cargo run --locked -- run examples/factorial.isk
```

The last command prints `3628800`. The current compiler handles one source file
at a time, checks a small `u64`/`bool` language, and uses Rust to produce native
executables. Function inspection and checked renames are working prototypes.
Modules, package resolution, and MCP are planned, not available. See the
[CLI reference](docs/cli.md) and [roadmap](ROADMAP.md).
If you use a coding agent, point it to [AGENTS.md](AGENTS.md) for
repository-specific guidance.

| Area | Where to start |
| --- | --- |
| Lexer, parser, type checking, Rust code generation, semantic queries | [`src/lib.rs`](src/lib.rs) |
| Commands and diagnostic output | [`src/main.rs`](src/main.rs) |
| CLI regression tests and inputs | [`tests/e2e.rs`](tests/e2e.rs), [`tests/fixtures/`](tests/fixtures/) |
| Runnable demonstrations and maintenance tasks | [`examples/`](examples/README.md), [controller showcase](showcase/README.md), [benchmarks](benchmarks/README.md) |

## Make a contribution

1. Check [existing issues](https://github.com/deed-ee/isk/issues) for related work.
   For significant changes, open an issue or, once enabled, a discussion before
   writing a large patch. Small fixes do not need a separate proposal.
2. Create a focused branch (`git switch -c my-change`) in your fork or clone.
   Make the change and add or update tests that demonstrate its behavior.
3. Run the checks below. Open a pull request explaining the problem, what
   changed, and how you verified it. Mention any checks you could not run.

```sh
cargo fmt --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
python3 benchmarks/pilot.py --task all --trials 1
```

For CLI work, `cargo test --locked --test e2e` exercises the real executable,
including examples and the controller. Keep changes small and test observable
behavior: check output and exit status for CLI changes, and prefer stable
diagnostic codes over exact wording unless the wording is part of the change.
The benchmark smoke check runs reference programs; it does not measure agent
productivity. See the [CI workflow](.github/workflows/ci.yml) for the full sequence.

## Propose a language change

Discuss major changes to syntax, semantics, types, ownership, effects,
concurrency, or modules before implementing them. Describe the problem, the
proposed behavior (with examples), alternatives, compatibility, and potential
effects on agent effort, safety, and performance. Explain how to test the
hypothesis and what would count against it. Use the
[design evaluation](docs/research/design-evaluation.md) and
[ADR template](docs/adr/template.md); unsettled choices remain provisional.

## AI-assisted contributions

Human-written and AI-assisted work are equally welcome. Use Copilot, Claude,
OpenCode, another tool, or none. You are responsible for what you submit:
understand and review generated changes, run relevant tests, and distinguish
verified results from hypotheses. Split large automated changes into reviewable
units. Give security-sensitive changes extra scrutiny, and never give agents
credentials or confidential reports.

## Where to communicate

- **Issues:** Reproducible bugs, scoped features, and design proposals. See
  [SECURITY.md](SECURITY.md) for suspected vulnerabilities.
- **Discussions, if enabled:** Open-ended questions, architecture ideas, and
  research or benchmark discussions. Until then, use an issue for a concrete
  proposal or question.
- **Pull requests:** Review a specific change, its tests, and its trade-offs.

Please follow the [Code of Conduct](CODE_OF_CONDUCT.md). The project uses the
[MIT License](LICENSE).
