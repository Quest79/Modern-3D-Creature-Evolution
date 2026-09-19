use serde::{Deserialize, Serialize};

use crate::{CreatureGenome, CreatureSimulator, CreatureSnapshot, SimulationConfig};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct FitnessWeights {
    pub distance: f32,
    pub average_speed: f32,
    pub upright: f32,
    pub stability: f32,
    /// Weight applied to average actuator mechanical-effort proxy.
    /// Use a negative value to penalize energy use.
    pub energy: f32,
}

impl Default for FitnessWeights {
    fn default() -> Self {
        Self {
            distance: 1.0,
            average_speed: 0.0,
            upright: 0.0,
            stability: 0.0,
            energy: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FitnessMetrics {
    pub distance: f32,
    pub average_speed: f32,
    /// 0 = sideways/upside-down, 1 = upright.
    pub upright: f32,
    /// 0..1 score based on root angular velocity. Higher is steadier.
    pub stability: f32,
    /// Average actuator effort per simulated second.
    pub energy: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct FitnessResult {
    pub score: f32,
    pub metrics: FitnessMetrics,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FitnessConfig {
    pub weights: FitnessWeights,
}

impl FitnessConfig {
    pub fn validate(&self) -> Result<(), String> {
        let weights = self.weights;
        for (name, value) in [
            ("distance", weights.distance),
            ("average_speed", weights.average_speed),
            ("upright", weights.upright),
            ("stability", weights.stability),
            ("energy", weights.energy),
        ] {
            if !value.is_finite() {
                return Err(format!("fitness weight {name} must be finite"));
            }
            if value.abs() > 10_000.0 {
                return Err(format!("fitness weight {name} magnitude is too large"));
            }
        }
        Ok(())
    }

    pub fn score(&self, metrics: FitnessMetrics) -> f32 {
        let weights = self.weights;
        weights.distance * metrics.distance
            + weights.average_speed * metrics.average_speed
            + weights.upright * metrics.upright
            + weights.stability * metrics.stability
            + weights.energy * metrics.energy
    }
}

pub fn evaluate_fitness(
    genome: &CreatureGenome,
    simulation: &SimulationConfig,
    fitness: &FitnessConfig,
) -> Result<FitnessResult, String> {
    simulation.validate()?;
    fitness.validate()?;
    genome.validate()?;

    let simulator = CreatureSimulator;
    let mut accumulator = FitnessAccumulator::default();

    let report = simulator.run_streaming(simulation, genome, 1, &mut |snapshot| {
        accumulator.observe(snapshot);
        Ok(())
    })?;

    let start = genome.segments[0].initial_position;
    let dx = report.final_root_position[0] - start[0];
    let dz = report.final_root_position[2] - start[2];
    let distance = (dx * dx + dz * dz).sqrt();

    let sample_count = accumulator.samples.max(1) as f32;
    let metrics = FitnessMetrics {
        distance,
        average_speed: accumulator.horizontal_speed_sum / sample_count,
        upright: accumulator.upright_sum / sample_count,
        stability: accumulator.stability_sum / sample_count,
        energy: if report.simulated_seconds > 0.0 {
            report.motor_effort / report.simulated_seconds
        } else {
            0.0
        },
    };

    Ok(FitnessResult {
        score: fitness.score(metrics),
        metrics,
    })
}

#[derive(Default)]
struct FitnessAccumulator {
    samples: usize,
    horizontal_speed_sum: f32,
    upright_sum: f32,
    stability_sum: f32,
}

impl FitnessAccumulator {
    fn observe(&mut self, snapshot: &CreatureSnapshot) {
        let Some(root) = snapshot.bodies.first() else {
            return;
        };

        let [vx, _vy, vz] = root.linear_velocity;
        self.horizontal_speed_sum += (vx * vx + vz * vz).sqrt();

        let [qx, qy, qz, qw] = root.rotation_xyzw;
        let norm_sq = qx * qx + qy * qy + qz * qz + qw * qw;
        let up_y = if norm_sq > 1.0e-8 {
            let inv_norm = norm_sq.sqrt().recip();
            let x = qx * inv_norm;
            let z = qz * inv_norm;
            1.0 - 2.0 * (x * x + z * z)
        } else {
            0.0
        };
        self.upright_sum += up_y.clamp(0.0, 1.0);

        let [wx, wy, wz] = root.angular_velocity;
        let angular_speed = (wx * wx + wy * wy + wz * wz).sqrt();
        self.stability_sum += 1.0 / (1.0 + angular_speed);

        self.samples += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{FitnessConfig, FitnessMetrics, FitnessWeights, evaluate_fitness};
    use crate::{CreatureGenome, SimulationConfig};

    #[test]
    fn weighted_score_combines_metrics() {
        let config = FitnessConfig {
            weights: FitnessWeights {
                distance: 2.0,
                average_speed: 3.0,
                upright: 4.0,
                stability: 5.0,
                energy: -0.5,
            },
        };
        let metrics = FitnessMetrics {
            distance: 1.0,
            average_speed: 2.0,
            upright: 0.5,
            stability: 0.25,
            energy: 4.0,
        };
        assert!((config.score(metrics) - 9.25).abs() < 1.0e-6);
    }

    #[test]
    fn default_fitness_matches_distance_metric() {
        let result = evaluate_fitness(
            &CreatureGenome::three_segment_walker(),
            &SimulationConfig {
                duration_seconds: 0.1,
                ..SimulationConfig::default()
            },
            &FitnessConfig::default(),
        )
        .unwrap();

        assert!((result.score - result.metrics.distance).abs() < 1.0e-6);
        assert!((0.0..=1.0).contains(&result.metrics.upright));
        assert!((0.0..=1.0).contains(&result.metrics.stability));
        assert!(result.metrics.energy >= 0.0);
    }
}
