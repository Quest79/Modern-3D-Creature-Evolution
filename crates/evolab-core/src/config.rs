use serde::{Deserialize, Serialize};

use crate::WorldConfig;

fn default_motor_strength_multiplier() -> f32 {
    1.0
}

/// Physics settings shared by every backend.
///
/// Fixed timesteps are a hard requirement for reproducible evolutionary runs.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SimulationConfig {
    /// Fixed physics timestep in seconds.
    pub dt: f32,
    /// Length of one evaluation in simulated seconds.
    pub duration_seconds: f32,
    /// Configurable terrain, obstacles, friction, gravity, and procedural seed.
    #[serde(default)]
    pub world: WorldConfig,
    /// Global actuator-strength scale applied during this simulation.
    #[serde(default = "default_motor_strength_multiplier")]
    pub motor_strength_multiplier: f32,
    /// Requests deterministic scheduling/stepping where the backend supports it.
    pub deterministic: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            dt: 1.0 / 120.0,
            duration_seconds: 5.0,
            world: WorldConfig::default(),
            motor_strength_multiplier: default_motor_strength_multiplier(),
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
        self.world.validate()?;
        if !self.motor_strength_multiplier.is_finite()
            || !(0.0..=1.0).contains(&self.motor_strength_multiplier)
        {
            return Err(
                "motor_strength_multiplier is biological activation and must be between 0 and 1"
                    .into(),
            );
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

    #[test]
    fn rejects_super_biological_motor_activation() {
        let config = SimulationConfig {
            motor_strength_multiplier: 1.01,
            ..SimulationConfig::default()
        };
        assert!(config.validate().is_err());
    }
}
