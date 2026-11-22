# ADR 0002: Explicit single-file test execution

**Status:** Provisional

**Hypothesis:** Named test roots and per-symbol results let agents verify a
change without writing a driver or inferring which function failed from
process output. The baseline is building and running a program with a
handwritten `main` that calls the desired checks.

**Decision:** Accept explicit zero-argument boolean roots from one checked
source snapshot. Build one temporary harness with the Rust backend. Run each
unique root in declaration order in its own process, reporting false returns
separately from abnormal exits. Keep normal builds and full CI unchanged.
Do not add test syntax, discovery, automatic selection, or cached binaries.

**Evidence and limits:** The CLI regression covers ordered, deduplicated
results for true, false, and panicking roots plus rejection of unknown or
invalid roots. The prototype has no execution timeout or sandbox. It makes no
claim about agent effort or runtime improvement. Before extending it, compare
driver-based and explicit-root runs on the same revision and report equivalent
outcomes, verification time, agent tokens, and correction cycles. For focused
verification, also measure tests missed relative to the full suite.
