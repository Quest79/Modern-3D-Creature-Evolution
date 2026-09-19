# Modern 3D Creature Evolution

A modern, hardware-accelerated virtual evolution platform inspired by 3D Virtual Creature Evolution (3DVCE) and later evolutionary robotics research.

The goal is to evolve complete virtual creatures — body morphology, joints, sensors, controllers, and behavior — inside configurable simulated worlds, using both classic genetic algorithms and newer evolutionary / quality-diversity methods.

See [SPEC.md](SPEC.md) for the full seven-part project specification.

## Development status

**Step 7 — Results / History / Analysis: implemented.**

Steps 1–7 are live. Evolution now persists per-generation history, champion archives, lineage, diversity, analysis classes, Pareto fronts, and MAP-Elites-style occupancy in a dedicated Results UI.

Current foundation:

- Rust simulation core independent of the renderer
- Rapier 3D CPU physics
- deterministic fixed-timestep execution
- parallel independent-world evaluation with Rayon
- replaceable physics-backend interface
- backend-neutral world-state snapshots
- real-time Rust → Godot state streaming
- progress and Stop/cancel controls
- capability reporting
- Godot 4 desktop GUI
- GitHub Actions CI
- multi-segment creature genomes
- revolute joints with limits and motor parameters
- deterministic seeded mutation
- structural add/remove-segment mutation
- random morphology generation
- JSON genome save/load
- mutate-current / random-creature controls in the GUI
- population evolution with tournament selection and elitism
- configurable fitness and champion tracking
- evolvable expression-tree brains stored inside each creature genome
- time, motion, and orientation sensors
- add/subtract/multiply/negate/sine/cosine/clamp brain nodes
- brain constant/expression/wrapper mutation
- joint outputs driven by the evolved brain every physics step
- configurable weighted fitness for distance, speed, uprightness, stability, and actuator effort
- champion fitness metric breakdown in the Godot GUI
- flat, slope, hills, and stairs terrain
- editable gravity and ground friction
- deterministic seeded world generation
- static walls, blocks, gaps, and pits
- saved world/terrain settings in the Godot GUI
- exact Rust-generated world geometry rendered in the Godot preview
- the same configured world used for evolution, creature playback, live probe runs, and benchmarks
- generation timeline keyframes for world, fitness, population, mutation, motor strength, duration, and trials
- conditional timeline events based on best fitness, average fitness, or best distance
- repeated creature trials with mean / median / worst / best aggregation
- saved human-readable .evo experiment files with load and fork support
- headless `evolab run experiment.evo` execution
- persistent .evoresults analysis files
- deterministic individual IDs and parent lineage tracking
- per-generation diversity statistics and analysis-class summaries
- distance/energy Pareto-front analysis
- MAP-Elites-style segment/brain occupancy analysis
- generation best/average fitness graph in the Godot Results window
- per-generation champion archive with load and comparison controls

The Rust backend remains the authoritative simulator. Godot renders streamed snapshots and provides creature, fitness, evolution, and
world/terrain controls while Rust remains authoritative for physics and world
geometry.

See [docs/STEP-1.md](docs/STEP-1.md), [docs/STEP-2.md](docs/STEP-2.md), [docs/STEP-3.md](docs/STEP-3.md), [docs/STEP-4.md](docs/STEP-4.md), [docs/STEP-5.md](docs/STEP-5.md), [docs/STEP-6.md](docs/STEP-6.md), and [docs/STEP-7.md](docs/STEP-7.md).

## Windows quick start

Run:

```text
BOOTSTRAP_AND_RUN.bat
```

The bootstrapper installs missing prerequisites, updates the repository, builds
the Rust backend, and launches the Godot GUI.

## Headless examples

```bash
cargo test --workspace
cargo run --release -p evolab-cli -- probe --batch 1000 --workers 12
cargo run --release -p evolab-cli -- capabilities
cargo run --release -p evolab-cli -- genome-generate --output creature.json --seed 42 --random-segments 6 --mutations 12
cargo run --release -p evolab-cli -- run FastWalker.evo --workers 24
```
