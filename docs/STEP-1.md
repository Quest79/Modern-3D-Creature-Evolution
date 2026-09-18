# Step 1 — Simulation Foundation and Execution Engine

## Goal

Establish the architecture that every later creature/evolution feature will use:

- simulation is independent from rendering
- fixed-timestep headless physics
- reproducible isolated worlds
- parallel CPU evaluation
- replaceable physics backend interface
- CLI execution without a GUI
- a clean boundary for the future Godot frontend
- CI that compiles, tests, formats, and lints the Rust workspace

## Implemented in this milestone

### Rust workspace

`evolab-core`
- backend-neutral simulation configuration
- `PhysicsBackend` trait
- Rapier 3D CPU backend
- deterministic fixed-step probe simulation
- Rayon worker-pool batch execution
- tests for config validation, repeatability, and parallel batches

`evolab-cli`
- `evolab probe` command
- configurable batch size
- configurable worker count
- configurable evaluation duration
- configurable fixed timestep
- throughput report

### UI boundary

`apps/godot/` establishes the location of the future Godot 4 frontend. It does
not contain physics or evolution logic.

## Try it

```bash
cargo run --release -p evolab-cli -- probe --batch 1000 --workers 12
```

A probe is currently just a rigid box dropped onto a flat ground plane. It is
deliberately boring: its purpose is to verify that independent worlds can be
simulated reproducibly and in parallel before creature morphology is added.

## Step 1 remaining work

The foundation is started, not finished. Before moving fully into Step 2:

1. add a formal world snapshot/state API
2. add backend capability reporting
3. add benchmark harnesses and physics-steps/s metrics
4. add cancellation and progress callbacks for long batches
5. add a backend registry/factory
6. define the FFI/IPC boundary used by Godot
7. add a CUDA batch-backend prototype behind a feature flag
8. add checkpoint-safe serialization of simulation inputs/results

The CPU backend is intentionally the reference implementation. Accelerator
backends must match its public contracts rather than leaking device-specific
details into evolution code.
