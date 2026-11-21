# Agent-native libraries and package intelligence

**Status: proposed.** The bootstrap checks one source file, requires `main`,
resolves calls in a flat function namespace, and assigns snapshot-local IDs.
There are no ISK modules, exports, packages, or dependency resolution yet.
The Rust dependencies in `Cargo.toml` build the compiler; they are not ISK
libraries or a trust signal for future ISK packages.

## First slice: local libraries

Prototype one application and one local library in the controller showcase.
Keep each module's private names local; consumers see only explicitly exported
signatures and import them by qualified name. Use existing scalar types and
functions. The library has no `main`; only the executable entry module needs
one. Start with an explicitly supplied set of workspace-relative module paths
and reject unresolved imports, duplicate module identities, paths outside
the workspace, and import cycles. No discovery by directory scanning, network
fetching, install hooks, or implicit access to host-language libraries.

First separate entry-point validation from module checking in `CheckedProgram`.
Then resolve modules and exports before checking consumer bodies. Use a
qualified key (workspace package, module path, export name) to describe a
public symbol, distinct from its file-local compiler ID. Do not assume the
key survives moves or renames. Keep the module graph and exported interfaces
in the shared semantic API. The backend may lower the first graph as one
Rust compilation unit; neither Rust module rules nor its ABI define ISK
visibility or linkage. External interoperability requires an explicit
boundary and separate capability and compatibility evaluation.

The first acceptance slice is a controller that calls a local library, plus
tests for private exports, ambiguous names, unresolved imports, and cycles.
Compare explicit module mapping with file-derived discovery and qualified
imports with unqualified imports before ratifying syntax. Compare separate
compilation only once interface changes and invalidation can be measured.

## Semantic interface data

Begin with a versioned, deterministic interface snapshot derived from checked
source: package and module identity, exported names and kinds, parameter and
result types, and resolved direct dependency edges. Return one module or
export on request rather than source bodies or the full graph. A local
workspace identity is not a publisher identity.

Extend the model as capabilities exist: contracts and invariants, inferred
effects and declared capabilities, compatibility constraints and supported
targets, test and coverage evidence, and origin, artifact digests, and
provenance. Label metadata by evidence: declared, compiler-derived, verified
against a policy, or unknown. Tie verification evidence to the exact artifact,
toolchain, target, and checks it covers. Missing effect or coverage data is
unknown, not an assertion of purity or completeness.

Discovery by capability, compatibility and conflict explanations, update
impact, and validated dependency changes belong to the same semantic API.
CLI and optional MCP adapters should expose only checked operations already
available in that API.

## Stages and trust boundaries

| Stage | Scope | Gate before expanding |
| --- | --- | --- |
| 1. Local modules and libraries | Explicit module graph and exports | Deterministic symbol resolution, visibility checks, and controller tests |
| 2. Multi-package workspaces | Explicit local dependency sources and repeatable resolution | Stable graph and conflict diagnostics regardless of enumeration order |
| 3. Versioned packages | Version constraints, lockfile, content and source identity | Locked builds reject missing or mismatched inputs instead of silently resolving again |
| 4. Secure distribution | Trusted publishing, verified provenance, advisory and SBOM support | Documented trust policy and isolated builds before consuming third-party artifacts |
| 5. Package intelligence | Semantic discovery, update plans, integration and compatibility verification | Measured agent benefit without weaker correctness or security |

Treat package metadata, documentation, archives, and cached artifacts as
untrusted input, not agent instructions. Relevant failures include wrong-source
or lookalike packages, modified artifacts, untrusted publishers, and builds
that access host resources. Pin dependency names to explicit sources, do not
fall back to another source on resolution failure, and make surprising source
changes visible in review. A checksum proves a match to an expected digest,
not who published it. Signatures and provenance need a configured trust policy;
advisories and SBOMs describe evidence, not absence of vulnerabilities.

Keep dependency inspection and resolution free of package code execution.
Before allowing third-party builds, evaluate OS-enforced filesystem, network,
credential, and process restrictions; a capability declaration alone enforces
nothing. Reproducibility requires pinned toolchains, targets, build inputs and
configuration in addition to a lockfile. Test compatibility and use broader
regressions when impact is uncertain. Measure cold and incremental builds before
adopting semantic or content-addressed caches; cache keys must cover the full
inputs used to produce the result.

No registry, version resolver, signing service, or package sandbox is provided
by the current bootstrap.
