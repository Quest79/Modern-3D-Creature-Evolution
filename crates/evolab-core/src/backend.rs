use serde::{Deserialize, Serialize};

use crate::SimulationConfig;

/// Minimal rigid-body probe used to validate physics backends before creature
/// morphology exists.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProbeSpec {
    pub initial_position: [f32; 3],
    pub half_extents: [f32; 3],
    pub restitution: f32,
    pub friction: f32,
}

impl Default for ProbeSpec {
    fn default() -> Self {
        Self {
            initial_position: [0.0, 3.0, 0.0],
            half_extents: [0.25, 0.25, 0.25],
            restitution: 0.15,
            friction: 0.8,
        }
    }
}

/// Backend-neutral snapshot of one simulated world's visible state.
///
/// Creature snapshots will later contain many bodies/joints, but the transport
/// and viewer can already be built against this stable shape.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WorldSnapshot {
    pub step: usize,
    pub simulated_seconds: f32,
    pub position: [f32; 3],
    /// Quaternion in x, y, z, w order.
    pub rotation_xyzw: [f32; 4],
    pub linear_velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
    pub sleeping: bool,
}

/// Backend-neutral result payload. Later creature evaluation reports will build
/// on this same pattern.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SimulationReport {
    pub backend: String,
    pub steps: usize,
    pub simulated_seconds: f32,
    pub final_position: [f32; 3],
    pub final_linear_velocity: [f32; 3],
}

/// Describes what a simulation backend can do without exposing backend-specific
/// types to the experiment/evolution layers.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BackendCapabilities {
    pub name: String,
    pub deterministic: bool,
    pub state_streaming: bool,
    pub parallel_worlds: bool,
    pub gpu_accelerated: bool,
}

/// Interface every physics backend must implement.
///
/// CPU Rapier is the reference implementation. CUDA and other accelerator
/// backends can be added without changing experiment/evolution code.
pub trait PhysicsBackend: Send + Sync {
    fn name(&self) -> &'static str;

    fn capabilities(&self) -> BackendCapabilities;

    fn run_probe(
        &self,
        config: &SimulationConfig,
        probe: &ProbeSpec,
    ) -> Result<SimulationReport, String>;

    fn run_probe_streaming(
        &self,
        config: &SimulationConfig,
        probe: &ProbeSpec,
        sample_every_steps: usize,
        observer: &mut dyn FnMut(&WorldSnapshot) -> Result<(), String>,
    ) -> Result<SimulationReport, String>;
}
