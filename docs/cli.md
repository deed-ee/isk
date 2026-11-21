# Bootstrap CLI reference

`isk check <file.isk>` validates source without invoking `rustc` or writing
build artifacts. `isk build <file.isk>` writes generated Rust and the native
executable to `.isk-build/` and prints the executable's path. `isk run
<file.isk>` builds and executes the program.

The bootstrap supports `u64`, `bool`, functions, local `let`/`mut` bindings,
arithmetic, comparisons, `if` expressions, calls, and `print` for integers and
booleans. Integer overflow and division by zero panic explicitly rather than
inheriting Rust profile-dependent overflow behavior. See the
[bootstrap ADR](adr/0001-mvp-language-subset.md) for language limits.

## Function inspection

`isk inspect <file.isk> <function>` checks the file and emits a tab-separated
`function <id> <start> <end> <signature>` record, then zero or more
`call <start> <end>` records in source order. Function bodies and unrelated
symbols are omitted. The start and exclusive end are zero-based UTF-8 byte
offsets of the name in the input. The ID is a zero-based declaration index,
not a persistent project-wide identifier. Calls to the builtin `print` do
not count as references to a declared function.

## Rename previews

`isk rename <file.isk> <old-name> <new-name>` changes the function declaration
and its resolved calls, validates the result, and prints it. The preview
does not edit the original. To save the preview, redirect to a **different**
path. The entry point `main`, the builtin `print`, and names already used by
another function are not valid rename targets.
