//! Core simulation and execution primitives for Modern 3D Creature Evolution.
//!
//! Step 1 deliberately keeps rendering out of this crate. The UI talks to the
//! simulation core instead of owning the physics loop.

mod backend;
mod batch;
mod config;
mod rapier_cpu;

pub use backend::{PhysicsBackend, ProbeSpec, SimulationReport};
pub use batch::BatchRunner;
pub use config::SimulationConfig;
pub use rapier_cpu::RapierCpuBackend;
