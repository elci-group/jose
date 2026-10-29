# Jose

Jose is a deterministic agent-management control plane. It profiles players,
classifies evidence exposure, compiles bounded role assignments, and records
replayable state transitions. It does not execute assigned work or invoke models.

## Phase I

The initial skeleton includes execution-specific player identity, typed capability
profiles and versioned roles, exposure policy with fail-closed handling, assignment
compilation, an event-backed player lifecycle, deterministic event replay, JSON
serialization types, a CLI, and mock-player fixtures.

Tov does not currently provide a package/import system, so Phase I keeps policy
semantics authoritative in typed Rust and includes a Tov policy example for
inspection. Tov is not used to mutate runtime state.

## Phase II — Managed execution contract

`compile_instruction` produces a stable role brief and includes only evidence
approved by `plan_exposure`. `record_outcome` accepts externally measured results;
`benchmark_summary` compares cold and managed outcomes for the same player and
task class, including closure, cost per verified close, latency, false closes,
regressions, and the number of shared pair IDs. Jose does not execute benchmark
tasks or claim causal significance from a small sample. The supplied benchmark
fixture is illustrative and is explicitly not an efficacy result.

## Commands

```sh
cargo test
cargo run -- version
cargo run -- mock-demo
cargo run -- benchmark-demo
tov check tov
```

Persistence, provider invocation, ELCI networking, statistical promotion, and a
statistically useful benchmark programme remain future work. Phase II supplies
the data contract and deterministic summary calculations needed by that work.
