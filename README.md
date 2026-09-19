# Modern 3D Creature Evolution

A modern, hardware-accelerated virtual evolution platform inspired by 3D Virtual Creature Evolution (3DVCE) and later evolutionary robotics research.

The goal is to evolve complete virtual creatures — body morphology, joints, sensors, controllers, and behavior — inside configurable simulated worlds, using both classic genetic algorithms and newer evolutionary / quality-diversity methods.

See [SPEC.md](SPEC.md) for the full seven-part project specification.

## Development status

**Step 2 — Creature Morphology: in progress.**

Step 1 is live and Step 2 now has a mutation-ready creature genome.

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

The Rust backend remains the authoritative simulator. Godot renders streamed
snapshots and now acts as a simple creature-generation/mutation editor.

See [docs/STEP-1.md](docs/STEP-1.md) and [docs/STEP-2.md](docs/STEP-2.md).

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
```
