# ADR 0003: Explicit single-entry project roots

**Provisional.** Previously `check` and `run` required a source file; users
had to know or discover the entry. Accept the scaffold's root as an input,
resolving just `src/main.isk`, with no manifest or source-tree search.

**Hypothesis:** on an unfamiliar scaffold, an agent can validate and run the
application without a file-discovery call. Acceptance: equivalent observable
program output on the starter and controller, unrelated files ignored,
missing or escaping entries rejected. Keep ordinary file mode and code
generation unchanged; root lookup is constant in the number of source files.

**Evidence:** CLI regression `explicit_project_roots_use_only_the_scaffold_entry`.
No agent time, token or discovery-call reduction has been measured.

**Provisional extension.** `build` and `inspect` now also accept explicit
roots, and an optional [`isk.toml`](../projects.md#optional-manifest) may
select a different single entry. The baseline remains `src/main.isk` with
no configuration. CLI regressions cover both modes, reject invalid
manifests without silently falling back, and reject entries outside the
root. Compare the total effort to locate and verify a non-default entry
with and without a manifest before expanding the format. No source-tree,
module or dependency discovery is performed.
