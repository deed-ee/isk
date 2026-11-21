# Security policy

ISK is an experimental bootstrap, not a production-hardened language or
compiler. There is no supported release line or promised response timeline.

## Report a vulnerability

GitHub private vulnerability reporting is not currently enabled, and no
alternate private reporting contact is published. Do not open a public issue
containing vulnerability details. Maintainers should enable private reporting
under **Settings > Code security** and update this policy once it is available.
After that, use **Security > Report a vulnerability** and include the affected
commit, platform, impact, and enough information to investigate privately.
Do not include credentials or unrelated private data.

## Current boundaries

- `isk check` analyzes source without executing the program or invoking the Rust
  compiler. The bootstrap checks a limited type system and detects integer
  overflow and division by zero at runtime.
- `isk build` runs the host's `rustc`; `isk run` launches the resulting native
  executable with the invoking user's permissions. Neither command provides a
  sandbox. Compile and execute untrusted input only in a suitably isolated
  environment.
- Rename previews do not write the source. Applying a plan checks the source
  snapshot and validates the replacement, but does not protect against all
  concurrent filesystem changes.

Capability enforcement, package provenance verification, and isolated builds
are design goals, not guarantees of the current implementation.
