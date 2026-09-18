# Modern 3D Creature Evolution

A modern, hardware-accelerated virtual evolution platform inspired by 3D Virtual Creature Evolution (3DVCE) and later evolutionary robotics research.

The goal is to evolve complete virtual creatures — body morphology, joints, sensors, controllers, and behavior — inside configurable simulated worlds, using both classic genetic algorithms and newer evolutionary / quality-diversity methods.

See [SPEC.md](SPEC.md) for the full seven-part project specification.

## Development status

**Step 1 — Simulation Foundation: live.**

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

The blue Step 1 probe can now be watched falling in real time while the Rust
backend remains the authoritative simulator.

See [docs/STEP-1.md](docs/STEP-1.md).

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
```
