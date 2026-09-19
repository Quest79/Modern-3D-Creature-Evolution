use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::{
    BrainContext, BrainGenome, JointSensorState, SegmentSensorState, SimulationConfig, WorldConfig,
    legacy_expression,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SegmentGene {
    pub id: u32,
    pub name: String,
    pub half_extents: [f32; 3],
    pub initial_position: [f32; 3],
    pub density: f32,
    pub friction: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct JointGene {
    pub parent_id: u32,
    pub child_id: u32,
    pub parent_anchor: [f32; 3],
    pub child_anchor: [f32; 3],
    pub axis: [f32; 3],
    pub limits_radians: [f32; 2],
    pub motor_amplitude_radians: f32,
    pub motor_frequency_hz: f32,
    pub motor_phase_radians: f32,
    pub motor_stiffness: f32,
    pub motor_damping: f32,
    pub motor_max_torque: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CreatureGenome {
    pub name: String,
    pub segments: Vec<SegmentGene>,
    pub joints: Vec<JointGene>,
    #[serde(default)]
    pub brain: BrainGenome,
}

impl CreatureGenome {
    pub fn three_segment_walker() -> Self {
        let mut creature = Self {
            name: "Three Segment Walker".to_string(),
            segments: vec![
                SegmentGene {
                    id: 0,
                    name: "torso".to_string(),
                    half_extents: [0.60, 0.25, 0.30],
                    initial_position: [0.0, 1.85, 0.0],
                    density: 1.0,
                    friction: 0.9,
                },
                SegmentGene {
                    id: 1,
                    name: "left_leg".to_string(),
                    half_extents: [0.15, 0.60, 0.15],
                    initial_position: [-0.38, 1.0, 0.0],
                    density: 1.0,
                    friction: 1.2,
                },
                SegmentGene {
                    id: 2,
                    name: "right_leg".to_string(),
                    half_extents: [0.15, 0.60, 0.15],
                    initial_position: [0.38, 1.0, 0.0],
                    density: 1.0,
                    friction: 1.2,
                },
            ],
            joints: vec![
                JointGene {
                    parent_id: 0,
                    child_id: 1,
                    parent_anchor: [-0.38, -0.25, 0.0],
                    child_anchor: [0.0, 0.60, 0.0],
                    axis: [0.0, 0.0, 1.0],
                    limits_radians: [-0.95, 0.95],
                    motor_amplitude_radians: 0.65,
                    motor_frequency_hz: 1.25,
                    motor_phase_radians: 0.0,
                    motor_stiffness: 32.0,
                    motor_damping: 4.5,
                    motor_max_torque: 18.0,
                },
                JointGene {
                    parent_id: 0,
                    child_id: 2,
                    parent_anchor: [0.38, -0.25, 0.0],
                    child_anchor: [0.0, 0.60, 0.0],
                    axis: [0.0, 0.0, 1.0],
                    limits_radians: [-0.95, 0.95],
                    motor_amplitude_radians: 0.65,
                    motor_frequency_hz: 1.25,
                    motor_phase_radians: std::f32::consts::PI,
                    motor_stiffness: 32.0,
                    motor_damping: 4.5,
                    motor_max_torque: 18.0,
                },
            ],
            brain: BrainGenome::default(),
        };
        creature.brain.sync_with_joints(&creature.joints);
        creature
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.segments.is_empty() {
            return Err("creature must contain at least one segment".into());
        }

        let mut ids = HashSet::new();
        for segment in &self.segments {
            if !ids.insert(segment.id) {
                return Err(format!("duplicate segment id {}", segment.id));
            }
            if segment
                .half_extents
                .iter()
                .any(|value| !value.is_finite() || *value <= 0.0)
            {
                return Err(format!(
                    "segment {} half_extents must be finite and greater than 0",
                    segment.id
                ));
            }
            if segment
                .initial_position
                .iter()
                .any(|value| !value.is_finite())
            {
                return Err(format!("segment {} position must be finite", segment.id));
            }
            if !segment.density.is_finite() || segment.density <= 0.0 {
                return Err(format!(
                    "segment {} density must be greater than 0",
                    segment.id
                ));
            }
            if !segment.friction.is_finite() || segment.friction < 0.0 {
                return Err(format!(
                    "segment {} friction must be non-negative",
                    segment.id
                ));
            }
        }

        for joint in &self.joints {
            if !ids.contains(&joint.parent_id) || !ids.contains(&joint.child_id) {
                return Err("joint references a missing segment".into());
            }
            if joint.parent_id == joint.child_id {
                return Err("joint cannot connect a segment to itself".into());
            }
            let axis_len_sq = joint.axis.iter().map(|value| value * value).sum::<f32>();
            if !axis_len_sq.is_finite() || axis_len_sq <= 1.0e-8 {
                return Err("joint axis must be non-zero and finite".into());
            }
            if joint.limits_radians[0] >= joint.limits_radians[1] {
                return Err("joint minimum limit must be less than maximum limit".into());
            }
        }

        self.brain.validate(&self.joints, &self.segments)?;
        Ok(())
    }
}

impl Default for CreatureGenome {
    fn default() -> Self {
        Self::three_segment_walker()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CreatureBodySnapshot {
    pub id: u32,
    pub name: String,
    pub half_extents: [f32; 3],
    pub position: [f32; 3],
    pub rotation_xyzw: [f32; 4],
    pub linear_velocity: [f32; 3],
    pub angular_velocity: [f32; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CreatureSnapshot {
    pub step: usize,
    pub simulated_seconds: f32,
    pub bodies: Vec<CreatureBodySnapshot>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CreatureReport {
    pub steps: usize,
    pub simulated_seconds: f32,
    pub final_root_position: [f32; 3],
    /// Approximate actuator mechanical work, integrated as |torque_proxy * angular_velocity| * dt.
    pub motor_effort: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CreatureSimulator;

impl CreatureSimulator {
    pub fn run_streaming(
        &self,
        config: &SimulationConfig,
        genome: &CreatureGenome,
        sample_every_steps: usize,
        observer: &mut dyn FnMut(&CreatureSnapshot) -> Result<(), String>,
    ) -> Result<CreatureReport, String> {
        config.validate()?;
        genome.validate()?;

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

        let mut handles: HashMap<u32, RigidBodyHandle> = HashMap::new();
        for segment in &genome.segments {
            let body = RigidBodyBuilder::dynamic()
                .translation(Vector::new(
                    segment.initial_position[0],
                    segment.initial_position[1],
                    segment.initial_position[2],
                ))
                .linear_damping(0.05)
                .angular_damping(0.05)
                .build();
            let body_handle = rigid_bodies.insert(body);

            let collider = ColliderBuilder::cuboid(
                segment.half_extents[0],
                segment.half_extents[1],
                segment.half_extents[2],
            )
            .density(segment.density)
            .friction(segment.friction)
            .build();
            colliders.insert_with_parent(collider, body_handle, &mut rigid_bodies);
            handles.insert(segment.id, body_handle);
        }

        let root_body_handle = *handles
            .get(&genome.segments[0].id)
            .ok_or_else(|| "missing root body handle".to_string())?;

        let mut impulse_joints = ImpulseJointSet::new();
        let mut motor_handles: Vec<(ImpulseJointHandle, JointGene)> = Vec::new();

        for joint_gene in &genome.joints {
            let parent = *handles
                .get(&joint_gene.parent_id)
                .ok_or_else(|| "missing parent body handle".to_string())?;
            let child = *handles
                .get(&joint_gene.child_id)
                .ok_or_else(|| "missing child body handle".to_string())?;

            let axis = Vector::new(joint_gene.axis[0], joint_gene.axis[1], joint_gene.axis[2]);

            let joint = RevoluteJointBuilder::new(axis)
                .local_anchor1(Vector::new(
                    joint_gene.parent_anchor[0],
                    joint_gene.parent_anchor[1],
                    joint_gene.parent_anchor[2],
                ))
                .local_anchor2(Vector::new(
                    joint_gene.child_anchor[0],
                    joint_gene.child_anchor[1],
                    joint_gene.child_anchor[2],
                ))
                .contacts_enabled(false)
                .limits(joint_gene.limits_radians)
                .motor_position(0.0, joint_gene.motor_stiffness, joint_gene.motor_damping)
                .motor_max_force(joint_gene.motor_max_torque)
                .build();

            let handle = impulse_joints.insert(parent, child, joint, true);
            motor_handles.push((handle, joint_gene.clone()));
        }

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
        let mut multibody_joints = MultibodyJointSet::new();
        let mut ccd_solver = CCDSolver::new();

        observer(&Self::snapshot(
            genome,
            &handles,
            &rigid_bodies,
            0,
            config.dt,
        )?)?;

        let sample_every_steps = sample_every_steps.max(1);
        let total_steps = config.step_count();
        let mut motor_effort = 0.0_f32;

        for step in 1..=total_steps {
            let time_seconds = (step - 1) as f32 * config.dt;
            let context = Self::brain_context(
                genome,
                &handles,
                &rigid_bodies,
                root_body_handle,
                time_seconds,
                &config.world,
            )?;

            for (joint_handle, gene) in &motor_handles {
                let target = genome
                    .brain
                    .output_for_joint(gene.child_id)
                    .map(|expression| expression.evaluate(&context))
                    .unwrap_or_else(|| legacy_expression(gene).evaluate(&context))
                    .clamp(gene.limits_radians[0], gene.limits_radians[1]);

                if let Some(sensor) = context
                    .joints
                    .iter()
                    .find(|sensor| sensor.child_id == gene.child_id)
                {
                    let position_error = (target - sensor.angle_radians).abs();
                    let angular_speed = sensor.velocity_radians_per_second.abs();
                    let torque_proxy = (position_error * gene.motor_stiffness
                        + angular_speed * gene.motor_damping)
                        .min(gene.motor_max_torque);
                    motor_effort += torque_proxy * angular_speed * config.dt;
                }

                if let Some(joint) = impulse_joints.get_mut(*joint_handle, true) {
                    joint.data.set_motor_position(
                        JointAxis::AngX,
                        target,
                        gene.motor_stiffness,
                        gene.motor_damping,
                    );
                    joint
                        .data
                        .set_motor_max_force(JointAxis::AngX, gene.motor_max_torque);
                }
            }

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
                observer(&Self::snapshot(
                    genome,
                    &handles,
                    &rigid_bodies,
                    step,
                    config.dt,
                )?)?;
            }
        }

        let root = rigid_bodies
            .get(root_body_handle)
            .ok_or_else(|| "root body disappeared".to_string())?;
        let p = root.translation();

        Ok(CreatureReport {
            steps: total_steps,
            simulated_seconds: total_steps as f32 * config.dt,
            final_root_position: [p.x, p.y, p.z],
            motor_effort,
        })
    }

    fn brain_context(
        genome: &CreatureGenome,
        handles: &HashMap<u32, RigidBodyHandle>,
        rigid_bodies: &RigidBodySet,
        root_body_handle: RigidBodyHandle,
        time_seconds: f32,
        world: &WorldConfig,
    ) -> Result<BrainContext, String> {
        let root = rigid_bodies
            .get(root_body_handle)
            .ok_or_else(|| "root body disappeared".to_string())?;
        let p = root.translation();
        let q = root.rotation();
        let v = root.linvel();
        let w = root.angvel();

        let mut joints = Vec::with_capacity(genome.joints.len());
        for gene in &genome.joints {
            let parent_handle = handles
                .get(&gene.parent_id)
                .ok_or_else(|| format!("missing parent body {}", gene.parent_id))?;
            let child_handle = handles
                .get(&gene.child_id)
                .ok_or_else(|| format!("missing child body {}", gene.child_id))?;
            let parent = rigid_bodies
                .get(*parent_handle)
                .ok_or_else(|| format!("parent body {} disappeared", gene.parent_id))?;
            let child = rigid_bodies
                .get(*child_handle)
                .ok_or_else(|| format!("child body {} disappeared", gene.child_id))?;

            let local_axis = Vector::new(gene.axis[0], gene.axis[1], gene.axis[2]).normalize();
            let relative_rotation = parent.rotation().inverse() * child.rotation();
            let angle = relative_rotation.to_scaled_axis().dot(local_axis);
            let world_axis = parent.rotation() * local_axis;
            let relative_angular_velocity = child.angvel() - parent.angvel();
            let velocity = relative_angular_velocity.dot(world_axis);

            joints.push(JointSensorState {
                child_id: gene.child_id,
                angle_radians: angle,
                velocity_radians_per_second: velocity,
            });
        }

        let mut segments = Vec::with_capacity(genome.segments.len());
        for segment in &genome.segments {
            let handle = handles
                .get(&segment.id)
                .ok_or_else(|| format!("missing body {}", segment.id))?;
            let body = rigid_bodies
                .get(*handle)
                .ok_or_else(|| format!("body {} disappeared", segment.id))?;

            let rotation = body.rotation();
            let axis_x = *rotation * Vector::X;
            let axis_y = *rotation * Vector::Y;
            let axis_z = *rotation * Vector::Z;
            let projected_half_height = axis_x.y.abs() * segment.half_extents[0]
                + axis_y.y.abs() * segment.half_extents[1]
                + axis_z.y.abs() * segment.half_extents[2];
            let bottom_y = body.translation().y - projected_half_height;
            let position = body.translation();
            let ground_contact = match world.surface_height_at(position.x, position.z) {
                Some(surface_y) if bottom_y <= surface_y + 0.03 => 1.0,
                _ => 0.0,
            };

            segments.push(SegmentSensorState {
                segment_id: segment.id,
                ground_contact,
            });
        }

        Ok(BrainContext {
            time_seconds,
            root_position: [p.x, p.y, p.z],
            root_linear_velocity: [v.x, v.y, v.z],
            root_angular_velocity: [w.x, w.y, w.z],
            root_rotation_xyzw: [q.x, q.y, q.z, q.w],
            joints,
            segments,
        })
    }

    fn snapshot(
        genome: &CreatureGenome,
        handles: &HashMap<u32, RigidBodyHandle>,
        rigid_bodies: &RigidBodySet,
        step: usize,
        dt: f32,
    ) -> Result<CreatureSnapshot, String> {
        let mut bodies = Vec::with_capacity(genome.segments.len());

        for segment in &genome.segments {
            let handle = handles
                .get(&segment.id)
                .ok_or_else(|| format!("missing handle for segment {}", segment.id))?;
            let body = rigid_bodies
                .get(*handle)
                .ok_or_else(|| format!("segment {} disappeared", segment.id))?;
            let p = body.translation();
            let q = body.rotation();
            let v = body.linvel();
            let w = body.angvel();

            bodies.push(CreatureBodySnapshot {
                id: segment.id,
                name: segment.name.clone(),
                half_extents: segment.half_extents,
                position: [p.x, p.y, p.z],
                rotation_xyzw: [q.x, q.y, q.z, q.w],
                linear_velocity: [v.x, v.y, v.z],
                angular_velocity: [w.x, w.y, w.z],
            });
        }

        Ok(CreatureSnapshot {
            step,
            simulated_seconds: step as f32 * dt,
            bodies,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{CreatureGenome, CreatureSimulator};
    use crate::SimulationConfig;

    #[test]
    fn default_genome_has_three_segments_and_two_joints() {
        let genome = CreatureGenome::default();
        assert_eq!(genome.segments.len(), 3);
        assert_eq!(genome.joints.len(), 2);
        assert!(genome.validate().is_ok());
    }

    #[test]
    fn creature_stream_contains_all_segments() {
        let simulator = CreatureSimulator;
        let genome = CreatureGenome::default();
        let mut seen_frames = 0;

        simulator
            .run_streaming(
                &SimulationConfig {
                    duration_seconds: 0.2,
                    ..SimulationConfig::default()
                },
                &genome,
                5,
                &mut |snapshot| {
                    assert_eq!(snapshot.bodies.len(), 3);
                    seen_frames += 1;
                    Ok(())
                },
            )
            .unwrap();

        assert!(seen_frames > 1);
    }
}
