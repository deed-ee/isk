# ISK language examples

Run a program from the repository root with `cargo run -- run examples/<file>.isk`.
The CLI integration suite compiles and executes each program below and checks
its output and exit status.

| Program | Demonstrates | Output |
| --- | --- | --- |
| [factorial.isk](factorial.isk) | Functions, recursion, multiplication | `3628800` |
| [coprime.isk](coprime.isk) | Euclid's algorithm, integer division, equality | `true` |
| [guarded-start.isk](guarded-start.isk) | Boolean guard and conditional branching | `false` |

For an application that combines these capabilities, see the
[controller showcase](../showcase/README.md#controller).
