# Step 1 — Simulation Foundation and Execution Engine

## Goal

Establish the architecture that every later creature/evolution feature will use:

- simulation independent from rendering
- fixed-timestep headless physics
- reproducible isolated worlds
- parallel CPU evaluation
- replaceable physics backend interface
- CLI execution without a GUI
- a Godot frontend that talks to the backend through a machine-readable boundary
- CI that compiles, tests, formats, and lints the Rust workspace

## Implemented

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
- human-readable throughput output
- `--json` machine-readable result protocol for GUI/automation clients
- worlds/s and physics-steps/s metrics

### Godot GUI

`apps/godot/` now contains a runnable Godot 4 frontend.

The GUI can:

- set number of independent worlds
- set CPU worker count
- set simulation duration
- set fixed physics timestep
- launch the Rust simulation without freezing the UI
- parse the backend's JSON result
- display performance metrics
- show the Step 1 probe in a basic 3D preview

Godot does not own the physics loop. The Rust executable remains the source of
simulation truth.

## Run it

On Windows, use:

```text
BOOTSTRAP_AND_RUN.bat
```

The bootstrap script installs/builds the required components and launches the GUI.

For headless testing:

```bash
cargo run --release -p evolab-cli -- probe --batch 1000 --workers 12
```

For the GUI protocol directly:

```bash
cargo run --release -p evolab-cli -- probe --batch 1000 --workers 12 --json
```

## Step 1 remaining work

1. formal world snapshot/state API
2. backend capability reporting
3. dedicated benchmark harness
4. cancellation and progress callbacks for long batches
5. backend registry/factory
6. persistent IPC or native FFI boundary if needed for high-frequency visualization
7. CUDA batch-backend prototype behind a feature flag
8. checkpoint-safe serialization of simulation inputs/results
9. stream intermediate world state to the viewer for true live playback

The CPU backend remains the reference implementation. Accelerator backends must
match its public contracts rather than leaking device-specific details into
evolution code.
