# isk
Agents for Agents

ISK is an experimental agent-native language. The current bootstrap compiler
implements a small, explicit subset and translates it to readable Rust before
invoking `rustc`.

## Try it

Install stable Rust, then run:

```sh
cargo run -- check examples/factorial.isk
cargo run -- run examples/factorial.isk
```

The program prints `3628800`. `build` writes generated Rust and the executable
to `.isk-build/`. The `isk` CLI currently supports `check`, `build`, and `run`;
other commands in the initial mandate are not implemented yet.

The MVP supports `u64`, `bool`, functions, local `let`/`mut` bindings, arithmetic,
comparisons, `if` expressions, calls, and `print` for integers and booleans.
Integer overflow and division by zero panic explicitly rather than inheriting
Rust profile-dependent overflow behavior. See
[`docs/adr/0001-mvp-language-subset.md`](docs/adr/0001-mvp-language-subset.md)
for current decisions and limitations.

## Development

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The workspace is currently a single package with internal compiler stages.
Semantic editing, contracts, effects, advanced types, JSON diagnostics, and the
agent-efficiency benchmark suite remain planned work; no benchmark results are
claimed. See the [roadmap](roadmap.md) for milestones and how we decide when
they are complete.
