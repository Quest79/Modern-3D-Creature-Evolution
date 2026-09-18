# Modern 3D Creature Evolution

A modern, hardware-accelerated virtual evolution platform inspired by 3D Virtual Creature Evolution (3DVCE) and later evolutionary robotics research.

The goal is to evolve complete virtual creatures — body morphology, joints, sensors, controllers, and behavior — inside configurable simulated worlds, using both classic genetic algorithms and newer evolutionary / quality-diversity methods.

See [SPEC.md](SPEC.md) for the full seven-part project specification.

## Development status

**Step 1 — Simulation Foundation: in progress.**

The project now has:

- headless Rust simulation core
- Rapier 3D CPU physics
- fixed-timestep deterministic execution
- parallel independent-world evaluation with Rayon
- replaceable physics-backend interface
- JSON output protocol for GUI/automation clients
- Godot 4 desktop GUI with basic 3D preview and simulation controls
- GitHub Actions CI

See [docs/STEP-1.md](docs/STEP-1.md).

## Windows quick start

Download/run:

```text
BOOTSTRAP_AND_RUN.bat
```

It installs missing prerequisites, updates the repository, builds the Rust backend,
and launches the Godot GUI.

## Headless quick start

```bash
cargo test --workspace
cargo run --release -p evolab-cli -- probe --batch 1000 --workers 12
```

Machine-readable output:

```bash
cargo run --release -p evolab-cli -- probe --batch 1000 --workers 12 --json
```
