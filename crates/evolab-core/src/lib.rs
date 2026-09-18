//! Core simulation and execution primitives for Modern 3D Creature Evolution.
//!
//! Rendering is intentionally outside this crate. The GUI consumes backend-
//! neutral snapshots/results and never owns the authoritative physics state.

mod backend;
mod batch;
mod config;
mod rapier_cpu;

pub use backend::{
    BackendCapabilities, PhysicsBackend, ProbeSpec, SimulationReport, WorldSnapshot,
};
pub use batch::BatchRunner;
pub use config::SimulationConfig;
pub use rapier_cpu::RapierCpuBackend;
