# Step 1 — Simulation Foundation and Execution Engine

## Goal

Establish the architecture that every later creature/evolution feature will use:

- simulation independent from rendering
- fixed-timestep headless physics
- reproducible isolated worlds
- parallel CPU evaluation
- replaceable physics backend interface
- backend-neutral world-state snapshots
- live state streaming to the Godot viewer
- progress and cancellation for long-running jobs
- backend capability reporting
- CI that compiles, tests, formats, and lints the Rust workspace

## Implemented

### Rust simulation core

`evolab-core`

- backend-neutral `SimulationConfig`
- `PhysicsBackend` interface
- backend capability reporting
- `WorldSnapshot` state API containing:
  - step
  - simulated time
  - position
  - rotation
  - linear velocity
  - angular velocity
  - sleeping state
- Rapier 3D CPU backend
- deterministic fixed-step simulation
- observer-based live state sampling
- Rayon worker-pool batch execution
- thread-safe progress callbacks
- tests for repeatability, streaming, and progress

### CLI/backend process

`evolab-cli`

- `probe`: parallel simulation benchmark
- `stream`: real-time single-world state streaming
- `capabilities`: machine/backend capability report
- versioned JSON event protocol
- UDP localhost events for:
  - batch start
  - batch progress
  - batch completion
  - live world state
  - live completion
- human-readable CLI mode remains available

### Godot GUI

The GUI now supports two different execution paths.

**Watch Live Physics**

Runs one authoritative physics world in Rust and streams state snapshots to
Godot at 60 Hz. The blue probe actually falls and settles in real time. Godot is
only visualizing the state; it is not re-simulating the body.

**Run Parallel Benchmark**

Evaluates the configured number of independent simulations using CPU workers.
The GUI shows completion progress, final throughput, and physics steps/second.

The GUI also provides:

- Stop button for either type of job
- backend/app version
- logical CPU-thread count
- deterministic/streaming/GPU capability status
- live height, speed, step, and simulated-time display

## Run it

On Windows:

```text
BOOTSTRAP_AND_RUN.bat
```

For headless benchmarking:

```bash
cargo run --release -p evolab-cli -- probe --batch 1000 --workers 12
```

Inspect capabilities:

```bash
cargo run --release -p evolab-cli -- capabilities
```

## Step 1 remaining work

1. dedicated benchmark suite and historical benchmark storage
2. backend registry/factory for multiple installed physics implementations
3. persistent IPC/native FFI if later creature visualization needs more bandwidth
4. CUDA batch-backend prototype behind a feature flag
5. checkpoint-safe serialization of simulation inputs/results
6. richer contact/joint state in world snapshots

The CPU backend remains the reference implementation. Accelerator backends must
match its public contracts rather than leaking device-specific details into
evolution code.

After these foundations are stable, Step 2 begins with the actual evolvable
creature-body genome: multiple segments, joints, motors, constraints, and
mutation-ready morphology.
