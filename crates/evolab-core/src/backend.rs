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

/// Interface every physics backend must implement.
///
/// CPU Rapier is the first implementation. CUDA and other accelerator backends
/// can be added without changing the experiment/evolution layers.
pub trait PhysicsBackend: Send + Sync {
    fn name(&self) -> &'static str;

    fn run_probe(
        &self,
        config: &SimulationConfig,
        probe: &ProbeSpec,
    ) -> Result<SimulationReport, String>;
}
