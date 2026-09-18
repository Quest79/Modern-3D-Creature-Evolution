use rapier3d::prelude::*;

use crate::{PhysicsBackend, ProbeSpec, SimulationConfig, SimulationReport};

/// General-purpose CPU physics backend.
///
/// Independent worlds are parallelized by `BatchRunner`; each individual world
/// remains isolated, which is the execution model we want for creature fitness
/// evaluation.
#[derive(Clone, Copy, Debug, Default)]
pub struct RapierCpuBackend;

impl PhysicsBackend for RapierCpuBackend {
    fn name(&self) -> &'static str {
        "rapier-cpu"
    }

    fn run_probe(
        &self,
        config: &SimulationConfig,
        probe: &ProbeSpec,
    ) -> Result<SimulationReport, String> {
        config.validate()?;

        if probe
            .half_extents
            .iter()
            .any(|v| !v.is_finite() || *v <= 0.0)
        {
            return Err("probe half_extents must be finite and greater than 0".into());
        }
        if !probe.restitution.is_finite() || !(0.0..=1.0).contains(&probe.restitution) {
            return Err("probe restitution must be between 0 and 1".into());
        }
        if !probe.friction.is_finite() || probe.friction < 0.0 {
            return Err("probe friction must be finite and non-negative".into());
        }

        let mut rigid_bodies = RigidBodySet::new();
        let mut colliders = ColliderSet::new();

        let ground = ColliderBuilder::cuboid(
            config.ground_half_extents[0],
            config.ground_half_extents[1],
            config.ground_half_extents[2],
        )
        .friction(0.9)
        .build();
        colliders.insert(ground);

        let body = RigidBodyBuilder::dynamic()
            .translation(Vector::new(
                probe.initial_position[0],
                probe.initial_position[1],
                probe.initial_position[2],
            ))
            .build();
        let body_handle = rigid_bodies.insert(body);

        let collider = ColliderBuilder::cuboid(
            probe.half_extents[0],
            probe.half_extents[1],
            probe.half_extents[2],
        )
        .restitution(probe.restitution)
        .friction(probe.friction)
        .build();
        colliders.insert_with_parent(collider, body_handle, &mut rigid_bodies);

        let gravity = Vector::new(config.gravity[0], config.gravity[1], config.gravity[2]);
        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.dt = config.dt;

        let mut physics_pipeline = PhysicsPipeline::new();
        let mut island_manager = IslandManager::new();
        let mut broad_phase = DefaultBroadPhase::new();
        let mut narrow_phase = NarrowPhase::new();
        let mut impulse_joints = ImpulseJointSet::new();
        let mut multibody_joints = MultibodyJointSet::new();
        let mut ccd_solver = CCDSolver::new();

        for _ in 0..config.step_count() {
            physics_pipeline.step(
                gravity,
                &integration_parameters,
                &mut island_manager,
                &mut broad_phase,
                &mut narrow_phase,
                &mut rigid_bodies,
                &mut colliders,
                &mut impulse_joints,
                &mut multibody_joints,
                &mut ccd_solver,
                &(),
                &(),
            );
        }

        let body = rigid_bodies
            .get(body_handle)
            .ok_or_else(|| "probe rigid body disappeared from simulation".to_string())?;

        let p = body.translation();
        let v = body.linvel();
        let steps = config.step_count();

        Ok(SimulationReport {
            backend: self.name().to_string(),
            steps,
            simulated_seconds: steps as f32 * config.dt,
            final_position: [p.x, p.y, p.z],
            final_linear_velocity: [v.x, v.y, v.z],
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{PhysicsBackend, ProbeSpec, RapierCpuBackend, SimulationConfig};

    #[test]
    fn falling_probe_reaches_ground() {
        let backend = RapierCpuBackend;
        let report = backend
            .run_probe(&SimulationConfig::default(), &ProbeSpec::default())
            .expect("probe simulation should succeed");

        // Ground top is y=0.1 and probe half-height is 0.25, so its center
        // should settle close to y=0.35.
        assert!((report.final_position[1] - 0.35).abs() < 0.08);
    }

    #[test]
    fn repeated_probe_is_reproducible() {
        let backend = RapierCpuBackend;
        let config = SimulationConfig::default();
        let probe = ProbeSpec::default();

        let a = backend.run_probe(&config, &probe).unwrap();
        let b = backend.run_probe(&config, &probe).unwrap();
        assert_eq!(a, b);
    }
}
