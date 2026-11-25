# ISK reference applications

Reference applications exercise real maintenance tasks, rather than illustrate
individual syntax features. They complement the small [language examples](../examples/README.md)
and the controlled [agent benchmarks](../benchmarks/README.md).

## Controller

[The controller](controller/src/main.isk) is an offline, deterministic model of an
automated system. It does not control hardware. It uses only `u64`, `bool`,
functions, and branches. `transition(state, event, interlock_clear)` is pure;
`main` feeds it a fixed event sequence and prints the resulting state IDs.

```sh
cargo run -- run showcase/controller
cargo test --locked --test e2e controller_showcase
```

The controller uses the [application layout](../docs/projects.md) with one
source file and a minimal manifest. The repository's CLI suite verifies its output.

| State | ID | Event | ID |
| --- | ---: | --- | ---: |
| Idle | 0 | Start | 0 |
| Starting | 1 | Ready | 1 |
| Running | 2 | Stop | 2 |
| Fault | 3 | Trip | 3 |
| Stopped | 4 | Reset | 4 |

- Start moves Idle or Stopped to Starting; Ready moves Starting to Running.
  Both require a clear interlock.
- Stop moves Starting or Running to Stopped without requiring a clear
  interlock.
- Trip enters Fault from any valid state. Fault is latched; only Reset with a
  clear interlock moves it to Stopped.
- An invalid state enters Fault. Other unmatched events leave the state unchanged.

## Capability stages

| Stage | Scope | Status |
| --- | --- | --- |
| A. State machine | Guarded transitions, fault handling, deterministic tests | Implemented |
| B. Event-driven app | Typed events, modules, error handling, state management | Planned |
| C. MQTT service | Serialization, network I/O, events, resource management | Planned |
| D. REST microservice | HTTP, JSON, validation, async execution, persistence | Planned |
| E. Larger system | Shared interfaces, dependencies, concurrency, iterative maintenance | Planned |

These are capability targets, not implementation commitments. When a task
exposes a missing capability, document the problem and expected benefit,
compare alternatives, and test the smallest viable change. Record correctness,
agent effort, and runtime trade-offs in an ADR before adopting a design.
The first maintenance tasks are in the [benchmark catalog](../benchmarks/controller-tasks.md).
