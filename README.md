# ISK — Agents for Agents

## Vision

**Build a programming language for agents, not merely with agents.** If
programming languages were invented for autonomous coding agents today, what
would we design differently? ISK is a language and development model built
around how agents understand, construct, transform, and verify software.

Human-readable source is one view of a program. Agents should be able to ask
the compiler what a symbol means, what depends on it, and which tests a change
could affect. They should be able to validate a structured change before it
touches the source, without reconstructing facts the compiler already knows.

Success means measurably less total effort to deliver correct, verified,
maintainable software without sacrificing safety, determinism, or native
performance. The current compiler translates a small provisional subset to
Rust. That is a bootstrap, not the language blueprint. See the
[architecture](docs/architecture.md), [agent instructions](AGENTS.md), and
[roadmap](ROADMAP.md).

## Try it

Install stable Rust, then run:

```sh
cargo run -- check examples/factorial.isk
cargo run -- build examples/factorial.isk
cargo run -- run examples/factorial.isk
```

`check` validates the source without invoking `rustc` or writing build
artifacts. `build` writes generated Rust and the executable to `.isk-build/`
and prints the executable's path. `run` builds and executes the program, which
prints `3628800` in this example.

The MVP supports `u64`, `bool`, functions, local `let`/`mut` bindings, arithmetic,
comparisons, `if` expressions, calls, and `print` for integers and booleans.
Integer overflow and division by zero panic explicitly rather than inheriting
Rust profile-dependent overflow behavior. See
[`docs/adr/0001-mvp-language-subset.md`](docs/adr/0001-mvp-language-subset.md)
for current decisions and limitations.

## Inspect or rename a function

```sh
cargo run -- inspect examples/factorial.isk factorial
cargo run -- rename examples/factorial.isk factorial fact
```

`inspect` checks the file, then emits tab-separated records for the requested
function and its resolved calls, with no function body or unrelated symbols:
`function <id> <start> <end> <signature>` followed by zero or more
`call <start> <end>` records in source order. Start and exclusive end are
zero-based UTF-8 byte offsets of the name in the input. The ID is a zero-based
declaration index, not a persistent project-wide identifier. Calls to the
builtin `print` do not count as references to a declared function.

`rename` validates the source, changes the function declaration and its resolved
calls, and validates the result before printing it. It never edits the input
file. To save the output, redirect it to a **different** path. The entry point
`main`, the builtin `print`, and names already used by another function are
not valid rename targets.

## Development

```sh
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 benchmarks/pilot.py --task all --trials 1
```

The workspace is currently a single package with internal compiler stages.
The numerical benchmark pilot can run the checked-in ISK, Rust, and Python
reference implementations with Python 3 and the same Rust toolchain:

```sh
python3 benchmarks/pilot.py --task all --trials 3 > pilot-results.jsonl
```

This measures the reference programs, not agent authoring. See the
[benchmark protocol](benchmarks/README.md) for the tasks, results format,
and measurement limits, and the [roadmap](ROADMAP.md) for planned work.

## License

ISK is available under the [MIT License](LICENSE). You can use, modify, and
redistribute it, including commercially, subject to the license notice
requirements.
