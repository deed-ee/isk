# Bootstrap CLI reference

`isk new <directory>` creates a starter application at `src/main.isk` and
prints its path. The destination must not already exist and its parent
must exist. Run the generated source with `isk run <path-to-main.isk>`.
See the [project model](projects.md) for the minimal manifest format.

`isk check <file.isk>` validates source without invoking `rustc` or writing
build artifacts. `isk build <file.isk>` writes generated Rust and the native
executable to `.isk-build/` and prints the executable's path. `isk run
<file.isk>` builds and executes the program.

`check`, `build`, `run`, and `inspect` also accept an explicitly supplied
directory containing `isk.toml`. They operate on that manifest's entry file.
Build output stays in `.isk-build/` relative to the current working directory.

The bootstrap supports `u64`, `bool`, functions, local `let`/`mut` bindings,
arithmetic, comparisons, `if` expressions, calls, and `print` for integers and
booleans. Integer overflow and division by zero panic explicitly rather than
inheriting Rust profile-dependent overflow behavior. See the
[bootstrap ADR](adr/0001-mvp-language-subset.md) for language limits.

## Structured diagnostics

`isk check <file.isk> --diagnostics=tsv` emits one tab-separated record to
stderr on a source error and exits with status 1:
`error <code> <line> <column> <path> <message>`. Line and Unicode scalar
column are one-based. Path and message escape backslashes and control
characters using Rust character escapes; paths that are not Unicode are
rendered lossily. Successful checks are silent in this mode. Usage and
file-read errors remain plain text.

## JSON output

Use `isk check <file.isk> --json` or `isk inspect <file.isk> <function> --json`.
On success, stdout contains one object with `schema_version: 1` and `ok: true`;
inspect adds `function` (signature and declaration range) and `calls` (byte
ranges and enclosing caller IDs/names, in source order). JSON inspection
includes caller details without `--callers`.

On source or symbol lookup errors, exit status is `1`, stdout is empty, and
stderr contains one JSON object with `schema_version`, `ok: false`, and
`error` with `code`, `message`, and `path`. Source errors also include
one-based `line` and Unicode scalar `column`. Usage and file-read errors
remain plain text; existing default and TSV output is unchanged.

## Function inspection

`isk inspect <file.isk> <function>` checks the file and emits a tab-separated
`function <id> <start> <end> <signature>` record, then zero or more
`call <start> <end>` records in source order. Function bodies and unrelated
symbols are omitted. The start and exclusive end are zero-based UTF-8 byte
offsets of the name in the input. The ID is a zero-based declaration index,
not a persistent project-wide identifier. Calls to the builtin `print` do
not count as references to a declared function.

`--callers` appends `<caller-id>` and `<caller-name>` as two tab-separated
fields on each `call` record. These are direct callers, including recursion.
A function appears more than once if it contains multiple calls.

## Rename previews

`isk rename <file.isk> <old-name> <new-name>` changes the function declaration
and its resolved calls, validates the result, and prints it. The preview
does not edit the original. To save the preview, redirect to a **different**
path. The entry point `main`, the builtin `print`, and names already used by
another function are not valid rename targets.

## Plan and apply

```sh
cp examples/factorial.isk work.isk
cargo run -- inspect work.isk factorial --callers
cargo run -- rename work.isk factorial fact --plan
snapshot='sha256:…' # Replace with the value printed by --plan.
cargo run -- rename work.isk factorial fact --apply "$snapshot"
cargo run -- check work.isk --diagnostics=tsv
cargo run -- run work.isk
```

The snapshot value is the second field of the plan's first record. The
remaining records are `replace <start> <end> <new-name>`, tab-separated,
in source order. Apply rebuilds and revalidates the requested rename,
rejects a mismatched snapshot, and atomically replaces a regular file. It
rejects symlinks and does not coordinate with concurrent external writers;
use it on a file you are not editing elsewhere. Run the full project test
suite when there is no reliable mapping from changed symbols to tests.

## Suggest affected tests (experimental)

`isk affected-tests <file.isk> <changed-function> [<test-root> ...]`
accepts one known body-only change and a caller-supplied inventory of test
entry functions in the checked file. The command does not compute a diff or
discover tests. For a rename, removed symbol, cross-file change, incomplete
inventory, or other uncertain impact, run the full suite instead.

On stdout, `scope\tfocused` is followed by `test\t<root>` records (one per
selected root in declaration order). Or the command emits
`scope\tfull\t<reason>` alone. Here `\t` denotes a tab. Reasons include
`no-roots`, `unknown-change`, `unknown-root`, and `no-reachable-root`.
Invalid source fails rather than returning a selection; use `check` for a
structured source diagnostic. The command never runs tests. A focused result
is a suggestion for early feedback, not a guarantee of correctness.
