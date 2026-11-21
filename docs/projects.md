# ISK application projects

An ISK project should tell an agent where to make a change and how to verify
it without requiring it to read the entire source tree. The project model
belongs to ISK, not Cargo, an IDE, or an AI platform.

## Working today

`isk new <directory>` creates a new directory with a runnable `src/main.isk`.
The parent must already exist; the command refuses existing destinations.
It does not create a manifest, lockfile, cache, or agent-specific prompt file.
Commands still take an explicit source file:

```sh
cargo run -- new my-application
cargo run -- check my-application/src/main.isk --json
cargo run -- run my-application/src/main.isk
```

The [controller](../showcase/controller/src/main.isk) uses the same layout
without a manifest. The CLI regression suite runs both the controller and a
generated starter.

## Proposed project format

```text
my-application/
├── isk.toml
├── src/
│   └── main.isk
└── README.md
```

The first manifest-aware application slice would recognize only:

```toml
[package]
name = "my-application"
version = "0.1.0"
entry = "src/main.isk"
```

`name` identifies the package locally, `version` is a declared semantic
version (not a resolver constraint), and `entry` is an optional relative
path defaulting to `src/main.isk`. Resolve it from the manifest directory
and reject missing entries or paths escaping that directory, including
resolved symlink targets. Report unknown keys and invalid values rather
than silently ignoring them. This is a proposal; the current CLI does not
read `isk.toml`. A README is recommended, and `AGENTS.md` is never a build
requirement.

Add application/library kinds once libraries can omit `main`; add explicit
local dependencies and members once module visibility and graph resolution
exist. Additional `.isk` modules under `src/`, tests under `tests/`, an
`isk.lock` for resolved dependencies, and a disposable `.isk/cache/` should
arrive only with their consumers. Targets, build configuration, capability
requirements, and verification policies should not become manifest fields
until the toolchain uses them. Follow the [package trust model](packages.md):
declared capabilities are not enforced permissions, and project discovery
must not run project or dependency code.

## Semantic API and CLI evolution

Separate project loading (project root, manifest, and source inputs) from
compiler analysis. Start with an explicitly supplied root, one entry, and a
checked snapshot. Extend it to a graph once module identities, public/private
visibility, library checking without `main`, and cross-module resolution are
available. Derive interfaces, types, call edges, and diagnostics from checked
source instead of duplicating them in a manifest. Keep symbol IDs snapshot
scoped until their stability across edits is defined.

Start project discovery with a compact summary: declared project identity,
available entry points, and validation commands. Add targeted queries for
dependencies, effects, contracts, test relationships, and change impact as the
compiler can substantiate them. Never report unknown information as absent.
Keep the core view independent of CLI, MCP, and the Rust backend.

Keep the current explicit file commands. Once project loading exists, design
a project-root mode for `check`, `build`, `run`, and `inspect`. Add `init`
only when there is meaningful metadata to initialize, `add`/`remove` only
with validated dependency edits, and `impact`, `test`, and `verify` only
when their underlying operations work. Start verification with deterministic
test discovery and structured diagnostics; a future `verify --affected` must
fall back to broader coverage when impact is uncertain. CI still runs full
regressions. Caches must not change results.

## Evaluation

Use the [benchmark protocol](../benchmarks/README.md) to compare equivalent
project creation, unfamiliar-project navigation, feature changes, debugging,
interface refactoring, dependency integration and updates, and verification
in ISK, Rust, and Python. Within ISK, compare semantic summaries against
normal file and documentation lookup on the same project revision. Record
total tokens, tool calls, time, corrections, and verified outcomes; report
failures. Do not infer a productivity gain from a working scaffold alone.
