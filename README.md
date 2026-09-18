# Modern 3D Creature Evolution

A modern, hardware-accelerated virtual evolution platform inspired by 3D Virtual Creature Evolution (3DVCE) and later evolutionary robotics research.

The goal is to evolve complete virtual creatures — body morphology, joints, sensors, controllers, and behavior — inside configurable simulated worlds, using both classic genetic algorithms and newer evolutionary / quality-diversity methods.

See [SPEC.md](SPEC.md) for the full seven-part project specification.

## Development status

**Step 1 — Simulation Foundation: started.**

The repository now has a headless Rust simulation core, a Rapier 3D CPU physics
backend, fixed-timestep execution, parallel independent-world evaluation with
Rayon, a CLI probe/benchmark command, a Godot frontend boundary, and CI.

See [docs/STEP-1.md](docs/STEP-1.md) for the current Step 1 implementation and
remaining work.

## Quick start

Install a current stable Rust toolchain, then:

```bash
cargo test --workspace
cargo run --release -p evolab-cli -- probe --batch 1000 --workers 12
```

The probe is the first end-to-end test of the architecture: many isolated 3D
physics worlds are evaluated headlessly and in parallel without any renderer.
