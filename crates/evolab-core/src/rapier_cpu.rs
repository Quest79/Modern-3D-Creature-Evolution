use rapier3d::prelude::*;

use crate::{
    BackendCapabilities, PhysicsBackend, ProbeSpec, SimulationConfig, SimulationReport,
    WorldSnapshot,
};

/// General-purpose CPU physics backend.
///
/// Independent worlds are parallelized by `BatchRunner`; each individual world
/// remains isolated, which is the execution model we want for creature fitness
/// evaluation.
#[derive(Clone, Copy, Debug, Default)]
pub struct RapierCpuBackend;

impl RapierCpuBackend {
    fn validate_probe(probe: &ProbeSpec) -> Result<(), String> {
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
        Ok(())
    }

    fn snapshot(body: &RigidBody, step: usize, dt: f32) -> WorldSnapshot {
        let p = body.translation();
        let q = body.rotation();
        let v = body.linvel();
        let w = body.angvel();

        WorldSnapshot {
            step,
            simulated_seconds: step as f32 * dt,
            position: [p.x, p.y, p.z],
            rotation_xyzw: [q.x, q.y, q.z, q.w],
            linear_velocity: [v.x, v.y, v.z],
            angular_velocity: [w.x, w.y, w.z],
            sleeping: body.is_sleeping(),
        }
    }

    fn simulate_probe(
        &self,
        config: &SimulationConfig,
        probe: &ProbeSpec,
        sample_every_steps: usize,
        observer: &mut dyn FnMut(&WorldSnapshot) -> Result<(), String>,
    ) -> Result<SimulationReport, String> {
        config.validate()?;
        Self::validate_probe(probe)?;

        let mut rigid_bodies = RigidBodySet::new();
        let mut colliders = ColliderSet::new();

        for shape in config.world.geometry() {
            let collider = ColliderBuilder::cuboid(
                shape.half_extents[0],
                shape.half_extents[1],
                shape.half_extents[2],
            )
            .translation(Vector::new(
                shape.center[0],
                shape.center[1],
                shape.center[2],
            ))
            .rotation(Vector::new(
                shape.rotation_radians[0],
                shape.rotation_radians[1],
                shape.rotation_radians[2],
            ))
            .friction(shape.friction)
            .build();
            colliders.insert(collider);
        }

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

        let gravity = Vector::new(
            config.world.gravity[0],
            config.world.gravity[1],
            config.world.gravity[2],
        );
        let integration_parameters = IntegrationParameters {
            dt: config.dt,
            ..IntegrationParameters::default()
        };

        let mut physics_pipeline = PhysicsPipeline::new();
        let mut island_manager = IslandManager::new();
        let mut broad_phase = DefaultBroadPhase::new();
        let mut narrow_phase = NarrowPhase::new();
        let mut impulse_joints = ImpulseJointSet::new();
        let mut multibody_joints = MultibodyJointSet::new();
        let mut ccd_solver = CCDSolver::new();

        let body = rigid_bodies
            .get(body_handle)
            .ok_or_else(|| "probe rigid body disappeared from simulation".to_string())?;
        observer(&Self::snapshot(body, 0, config.dt))?;

        let total_steps = config.step_count();
        let sample_every_steps = sample_every_steps.max(1);

        for step in 1..=total_steps {
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

            if step % sample_every_steps == 0 || step == total_steps {
                let body = rigid_bodies
                    .get(body_handle)
                    .ok_or_else(|| "probe rigid body disappeared from simulation".to_string())?;
                observer(&Self::snapshot(body, step, config.dt))?;
            }
        }

        let body = rigid_bodies
            .get(body_handle)
            .ok_or_else(|| "probe rigid body disappeared from simulation".to_string())?;

        let p = body.translation();
        let v = body.linvel();

        Ok(SimulationReport {
            backend: self.name().to_string(),
            steps: total_steps,
            simulated_seconds: total_steps as f32 * config.dt,
            final_position: [p.x, p.y, p.z],
            final_linear_velocity: [v.x, v.y, v.z],
        })
    }
}

impl PhysicsBackend for RapierCpuBackend {
    fn name(&self) -> &'static str {
        "rapier-cpu"
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            name: self.name().to_string(),
            deterministic: true,
            state_streaming: true,
            parallel_worlds: true,
            gpu_accelerated: false,
        }
    }

    fn run_probe(
        &self,
        config: &SimulationConfig,
        probe: &ProbeSpec,
    ) -> Result<SimulationReport, String> {
        let mut ignore = |_snapshot: &WorldSnapshot| Ok(());
        self.simulate_probe(config, probe, usize::MAX, &mut ignore)
    }

    fn run_probe_streaming(
        &self,
        config: &SimulationConfig,
        probe: &ProbeSpec,
        sample_every_steps: usize,
        observer: &mut dyn FnMut(&WorldSnapshot) -> Result<(), String>,
    ) -> Result<SimulationReport, String> {
        self.simulate_probe(config, probe, sample_every_steps, observer)
    }
}

#[cfg(test)]
mod tests {
    use crate::{PhysicsBackend, ProbeSpec, RapierCpuBackend, SimulationConfig, WorldSnapshot};

    #[test]
    fn falling_probe_reaches_ground() {
        let backend = RapierCpuBackend;
        let report = backend
            .run_probe(&SimulationConfig::default(), &ProbeSpec::default())
            .expect("probe simulation should succeed");

        // Default ground surface is y=0.0 and probe half-height is 0.25.
        assert!((report.final_position[1] - 0.25).abs() < 0.08);
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

    #[test]
    fn streaming_includes_initial_and_final_state() {
        let backend = RapierCpuBackend;
        let config = SimulationConfig::default();
        let mut frames: Vec<WorldSnapshot> = Vec::new();

        backend
            .run_probe_streaming(&config, &ProbeSpec::default(), 60, &mut |snapshot| {
                frames.push(snapshot.clone());
                Ok(())
            })
            .unwrap();

        assert_eq!(frames.first().unwrap().step, 0);
        assert_eq!(frames.last().unwrap().step, config.step_count());
        assert!(frames.len() > 2);
    }
}
