use serde::{Deserialize, Serialize};

/// Physics settings shared by every backend.
///
/// Fixed timesteps are a hard requirement for reproducible evolutionary runs.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SimulationConfig {
    /// Fixed physics timestep in seconds.
    pub dt: f32,
    /// Length of one evaluation in simulated seconds.
    pub duration_seconds: f32,
    /// World gravity in m/s^2.
    pub gravity: [f32; 3],
    /// Half-extents of the default flat test ground.
    pub ground_half_extents: [f32; 3],
    /// Requests deterministic scheduling/stepping where the backend supports it.
    pub deterministic: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            dt: 1.0 / 120.0,
            duration_seconds: 5.0,
            gravity: [0.0, -9.81, 0.0],
            ground_half_extents: [50.0, 0.1, 50.0],
            deterministic: true,
        }
    }
}

impl SimulationConfig {
    pub fn validate(&self) -> Result<(), String> {
        if !self.dt.is_finite() || self.dt <= 0.0 {
            return Err("dt must be finite and greater than 0".into());
        }
        if !self.duration_seconds.is_finite() || self.duration_seconds <= 0.0 {
            return Err("duration_seconds must be finite and greater than 0".into());
        }
        if self.gravity.iter().any(|v| !v.is_finite()) {
            return Err("gravity must contain only finite values".into());
        }
        if self
            .ground_half_extents
            .iter()
            .any(|v| !v.is_finite() || *v <= 0.0)
        {
            return Err("ground_half_extents must be finite and greater than 0".into());
        }
        Ok(())
    }

    pub fn step_count(&self) -> usize {
        (self.duration_seconds / self.dt).ceil() as usize
    }
}

#[cfg(test)]
mod tests {
    use super::SimulationConfig;

    #[test]
    fn default_config_is_valid() {
        let config = SimulationConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.step_count(), 600);
    }

    #[test]
    fn rejects_zero_timestep() {
        let config = SimulationConfig {
            dt: 0.0,
            ..SimulationConfig::default()
        };
        assert!(config.validate().is_err());
    }
}
