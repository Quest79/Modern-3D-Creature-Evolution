use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::{
    BrainContext, BrainGenome, JointSensorState, SegmentSensorState, SimulationConfig, WorldConfig,
    legacy_expression,
};

/// Broad observed range for dense biological structural materials represented as
/// bulk segment density. The low end covers very porous woods such as balsa;
/// the high end covers highly mineralized tissues such as enamel.
pub const BIOLOGICAL_MIN_DENSITY_KG_M3: f32 = 40.0;
pub const BIOLOGICAL_MAX_DENSITY_KG_M3: f32 = 3_000.0;
pub const BIOLOGICAL_MAX_CONTACT_FRICTION: f32 = 2.1;

/// Upper envelopes for direct biological muscle actuation. The stress ceiling
/// intentionally includes unusually strong invertebrate muscle; the power
/// ceiling is the highest measured cycle-average muscle power scale.
pub const BIOLOGICAL_MAX_MUSCLE_STRESS_PA: f32 = 1_400_000.0;
pub const BIOLOGICAL_MAX_CYCLIC_POWER_W_PER_KG: f32 = 400.0;
pub const MUSCLE_DENSITY_KG_M3: f32 = 1_060.0;
/// Hard validity ceiling for any creature body segment's linear speed.
pub const MAX_VALID_LINEAR_SPEED_M_S: f32 = 100.0;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BiologicalMaterial {
    PorousPlant,
    DensePlant,
    Adipose,
    #[default]
    SoftTissue,
    FibrousTissue,
    TrabecularBone,
    CorticalBone,
    Exoskeleton,
    MineralizedTissue,
}

impl BiologicalMaterial {
    pub fn density_range_kg_m3(self) -> (f32, f32) {
        match self {
            Self::PorousPlant => (40.0, 400.0),
            Self::DensePlant => (400.0, 1_400.0),
            Self::Adipose => (900.0, 1_000.0),
            Self::SoftTissue => (1_000.0, 1_150.0),
            Self::FibrousTissue => (1_050.0, 1_300.0),
            Self::TrabecularBone => (400.0, 1_600.0),
            Self::CorticalBone => (1_800.0, 2_200.0),
            Self::Exoskeleton => (1_000.0, 1_300.0),
            Self::MineralizedTissue => (2_000.0, 3_000.0),
        }
    }

    pub fn representative_density_kg_m3(self) -> f32 {
        match self {
            Self::PorousPlant => 180.0,
            Self::DensePlant => 700.0,
            Self::Adipose => 950.0,
            Self::SoftTissue => 1_050.0,
            Self::FibrousTissue => 1_130.0,
            Self::TrabecularBone => 800.0,
            Self::CorticalBone => 2_000.0,
            Self::Exoskeleton => 1_150.0,
            Self::MineralizedTissue => 2_500.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SegmentGene {
    pub id: u32,
    pub name: String,
    pub half_extents: [f32; 3],
    pub initial_position: [f32; 3],
    #[serde(default)]
    pub material: BiologicalMaterial,
    pub density: f32,
    pub friction: f32,
}

impl SegmentGene {
    pub fn volume_m3(&self) -> f32 {
        8.0 * self.half_extents[0] * self.half_extents[1] * self.half_extents[2]
    }

    pub fn mass_kg(&self) -> f32 {
        self.volume_m3() * self.density
    }

    fn contractile_cross_section_and_lever(&self) -> (f32, f32) {
        let longest_axis = (0..3)
            .max_by(|&a, &b| self.half_extents[a].total_cmp(&self.half_extents[b]))
            .unwrap_or(0);
        let transverse: Vec<usize> = (0..3).filter(|axis| *axis != longest_axis).collect();
        let a = transverse[0];
        let b = transverse[1];
        let area_m2 = (2.0 * self.half_extents[a]) * (2.0 * self.half_extents[b]);
        let lever_m = self.half_extents[a].min(self.half_extents[b]);
        (area_m2, lever_m)
    }
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
    /// Legacy serialized field. Runtime PD stiffness is derived from joint
    /// inertia, range of motion, and the current biological torque ceiling.
    #[serde(default)]
    pub motor_stiffness: f32,
    /// Legacy serialized field. Runtime PD damping is derived for critical
    /// damping from the physical stiffness and effective joint inertia.
    #[serde(default)]
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
                    half_extents: [0.30, 0.125, 0.15],
                    initial_position: [0.0, 0.735, 0.0],
                    material: BiologicalMaterial::SoftTissue,
                    density: 1_050.0,
                    friction: 0.6,
                },
                SegmentGene {
                    id: 1,
                    name: "left_leg".to_string(),
                    half_extents: [0.075, 0.30, 0.075],
                    initial_position: [-0.19, 0.31, 0.0],
                    material: BiologicalMaterial::SoftTissue,
                    density: 1_050.0,
                    friction: 1.2,
                },
                SegmentGene {
                    id: 2,
                    name: "right_leg".to_string(),
                    half_extents: [0.075, 0.30, 0.075],
                    initial_position: [0.19, 0.31, 0.0],
                    material: BiologicalMaterial::SoftTissue,
                    density: 1_050.0,
                    friction: 1.2,
                },
            ],
            joints: vec![
                JointGene {
                    parent_id: 0,
                    child_id: 1,
                    parent_anchor: [-0.19, -0.125, 0.0],
                    child_anchor: [0.0, 0.30, 0.0],
                    axis: [0.0, 0.0, 1.0],
                    limits_radians: [-0.95, 0.95],
                    motor_amplitude_radians: 0.65,
                    motor_frequency_hz: 1.25,
                    motor_phase_radians: 0.0,
                    motor_stiffness: 0.0,
                    motor_damping: 0.0,
                    motor_max_torque: 18.0,
                },
                JointGene {
                    parent_id: 0,
                    child_id: 2,
                    parent_anchor: [0.19, -0.125, 0.0],
                    child_anchor: [0.0, 0.30, 0.0],
                    axis: [0.0, 0.0, 1.0],
                    limits_radians: [-0.95, 0.95],
                    motor_amplitude_radians: 0.65,
                    motor_frequency_hz: 1.25,
                    motor_phase_radians: std::f32::consts::PI,
                    motor_stiffness: 0.0,
                    motor_damping: 0.0,
                    motor_max_torque: 18.0,
                },
            ],
            brain: BrainGenome::default(),
        };
        let torque_limits: Vec<f32> = creature
            .joints
            .iter()
            .map(|joint| {
                creature
                    .biological_joint_limits(joint)
                    .map(|(torque, _)| torque * 0.08)
                    .unwrap_or(0.0)
            })
            .collect();
        for (joint, torque_limit) in creature.joints.iter_mut().zip(torque_limits) {
            joint.motor_max_torque = torque_limit;
        }

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
            if !segment.density.is_finite()
                || !(BIOLOGICAL_MIN_DENSITY_KG_M3..=BIOLOGICAL_MAX_DENSITY_KG_M3)
                    .contains(&segment.density)
            {
                return Err(format!(
                    "segment {} density must be between {} and {} kg/m^3",
                    segment.id, BIOLOGICAL_MIN_DENSITY_KG_M3, BIOLOGICAL_MAX_DENSITY_KG_M3
                ));
            }
            let (material_min_density, material_max_density) =
                segment.material.density_range_kg_m3();
            if !(material_min_density..=material_max_density).contains(&segment.density) {
                return Err(format!(
                    "segment {} density {:.1} kg/m^3 is outside {:?} range {:.1}..={:.1} kg/m^3",
                    segment.id,
                    segment.density,
                    segment.material,
                    material_min_density,
                    material_max_density
                ));
            }
            if !segment.friction.is_finite()
                || !(0.0..=BIOLOGICAL_MAX_CONTACT_FRICTION).contains(&segment.friction)
            {
                return Err(format!(
                    "segment {} friction must be between 0 and {}",
                    segment.id, BIOLOGICAL_MAX_CONTACT_FRICTION
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

            let parent = self
                .segments
                .iter()
                .find(|segment| segment.id == joint.parent_id)
                .ok_or_else(|| "joint parent segment is missing".to_string())?;
            let child = self
                .segments
                .iter()
                .find(|segment| segment.id == joint.child_id)
                .ok_or_else(|| "joint child segment is missing".to_string())?;

            let joint_scale_m = parent
                .half_extents
                .iter()
                .chain(child.half_extents.iter())
                .copied()
                .fold(0.0_f32, f32::max)
                .max(1.0e-4);
            let anchor_tolerance_m = (joint_scale_m * 1.0e-5).max(1.0e-7);

            for axis in 0..3 {
                if !joint.parent_anchor[axis].is_finite()
                    || joint.parent_anchor[axis].abs()
                        > parent.half_extents[axis] + anchor_tolerance_m
                {
                    return Err(format!(
                        "joint {}→{} parent anchor lies outside segment {}",
                        joint.parent_id, joint.child_id, parent.id
                    ));
                }
                if !joint.child_anchor[axis].is_finite()
                    || joint.child_anchor[axis].abs()
                        > child.half_extents[axis] + anchor_tolerance_m
                {
                    return Err(format!(
                        "joint {}→{} child anchor lies outside segment {}",
                        joint.parent_id, joint.child_id, child.id
                    ));
                }
            }

            let anchor_error_sq = (0..3)
                .map(|axis| {
                    let parent_world = parent.initial_position[axis] + joint.parent_anchor[axis];
                    let child_world = child.initial_position[axis] + joint.child_anchor[axis];
                    let error = parent_world - child_world;
                    error * error
                })
                .sum::<f32>();
            if anchor_error_sq > anchor_tolerance_m.powi(2) {
                return Err(format!(
                    "joint {}→{} anchors do not coincide at spawn",
                    joint.parent_id, joint.child_id
                ));
            }

            let axis_len_sq = joint.axis.iter().map(|value| value * value).sum::<f32>();
            if !axis_len_sq.is_finite() || axis_len_sq <= 1.0e-8 {
                return Err("joint axis must be non-zero and finite".into());
            }
            if joint.limits_radians.iter().any(|value| !value.is_finite())
                || joint.limits_radians[0] >= joint.limits_radians[1]
            {
                return Err("joint limits must be finite and increasing".into());
            }
            if joint.limits_radians[1] - joint.limits_radians[0] > std::f32::consts::PI {
                return Err("revolute biological joint span must not exceed 180 degrees".into());
            }
            if !joint.motor_amplitude_radians.is_finite()
                || joint.motor_amplitude_radians < 0.0
                || !joint.motor_frequency_hz.is_finite()
                || joint.motor_frequency_hz < 0.0
                || !joint.motor_phase_radians.is_finite()
                || !joint.motor_stiffness.is_finite()
                || joint.motor_stiffness < 0.0
                || !joint.motor_damping.is_finite()
                || joint.motor_damping < 0.0
                || !joint.motor_max_torque.is_finite()
                || joint.motor_max_torque < 0.0
            {
                return Err("joint motor parameters must be finite and non-negative".into());
            }
        }

        self.brain.validate(&self.joints, &self.segments)?;
        Ok(())
    }

    /// Returns generous biological upper bounds for joint torque and continuous
    /// actuator power from the geometry of the two connected segments.
    ///
    /// The torque bound assumes the entire limiting transverse section could be
    /// active contractile tissue, so it is deliberately an upper envelope rather
    /// than a human-specific value.
    pub fn biological_joint_limits(&self, joint: &JointGene) -> Result<(f32, f32), String> {
        let parent = self
            .segments
            .iter()
            .find(|segment| segment.id == joint.parent_id)
            .ok_or_else(|| format!("missing parent segment {}", joint.parent_id))?;
        let child = self
            .segments
            .iter()
            .find(|segment| segment.id == joint.child_id)
            .ok_or_else(|| format!("missing child segment {}", joint.child_id))?;

        let (parent_area, parent_lever) = parent.contractile_cross_section_and_lever();
        let (child_area, child_lever) = child.contractile_cross_section_and_lever();
        let torque_limit = BIOLOGICAL_MAX_MUSCLE_STRESS_PA
            * (parent_area * parent_lever).min(child_area * child_lever);

        let parent_contractile_mass = parent
            .mass_kg()
            .min(parent.volume_m3() * MUSCLE_DENSITY_KG_M3);
        let child_contractile_mass = child
            .mass_kg()
            .min(child.volume_m3() * MUSCLE_DENSITY_KG_M3);
        let power_limit = BIOLOGICAL_MAX_CYCLIC_POWER_W_PER_KG
            * parent_contractile_mass.min(child_contractile_mass);

        Ok((torque_limit.max(0.0), power_limit.max(0.0)))
    }

    /// Effective rotational inertia seen by a revolute joint. This is computed
    /// from each cuboid's mass distribution plus the parallel-axis contribution
    /// from its centre of mass to the joint anchor.
    pub fn joint_effective_inertia(&self, joint: &JointGene) -> Result<f32, String> {
        let parent = self
            .segments
            .iter()
            .find(|segment| segment.id == joint.parent_id)
            .ok_or_else(|| format!("missing parent segment {}", joint.parent_id))?;
        let child = self
            .segments
            .iter()
            .find(|segment| segment.id == joint.child_id)
            .ok_or_else(|| format!("missing child segment {}", joint.child_id))?;

        let axis_length = joint
            .axis
            .iter()
            .map(|value| value * value)
            .sum::<f32>()
            .sqrt();
        if !axis_length.is_finite() || axis_length <= 1.0e-8 {
            return Err("joint axis must be non-zero and finite".into());
        }
        let axis = [
            joint.axis[0] / axis_length,
            joint.axis[1] / axis_length,
            joint.axis[2] / axis_length,
        ];

        let parent_inertia = Self::segment_inertia_about_anchor(parent, joint.parent_anchor, axis);
        let child_inertia = Self::segment_inertia_about_anchor(child, joint.child_anchor, axis);
        let sum = parent_inertia + child_inertia;
        if !sum.is_finite() || sum <= 1.0e-12 {
            return Err("joint effective inertia must be finite and positive".into());
        }

        Ok((parent_inertia * child_inertia / sum).max(1.0e-12))
    }

    fn segment_inertia_about_anchor(
        segment: &SegmentGene,
        anchor: [f32; 3],
        axis: [f32; 3],
    ) -> f32 {
        let mass = segment.mass_kg();
        let [hx, hy, hz] = segment.half_extents;
        let ixx = mass / 3.0 * (hy * hy + hz * hz);
        let iyy = mass / 3.0 * (hx * hx + hz * hz);
        let izz = mass / 3.0 * (hx * hx + hy * hy);
        let inertia_about_com =
            axis[0] * axis[0] * ixx + axis[1] * axis[1] * iyy + axis[2] * axis[2] * izz;

        let r_squared = anchor.iter().map(|value| value * value).sum::<f32>();
        let r_dot_axis = anchor[0] * axis[0] + anchor[1] * axis[1] + anchor[2] * axis[2];
        let perpendicular_distance_squared = (r_squared - r_dot_axis * r_dot_axis).max(0.0);

        inertia_about_com + mass * perpendicular_distance_squared
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
                .linear_damping(0.0)
                .angular_damping(0.0)
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
        let mut motor_handles: Vec<(ImpulseJointHandle, JointGene, f32, f32, f32)> = Vec::new();

        for joint_gene in &genome.joints {
            let parent = *handles
                .get(&joint_gene.parent_id)
                .ok_or_else(|| "missing parent body handle".to_string())?;
            let child = *handles
                .get(&joint_gene.child_id)
                .ok_or_else(|| "missing child body handle".to_string())?;

            let axis = Vector::new(joint_gene.axis[0], joint_gene.axis[1], joint_gene.axis[2]);

            let (biological_torque_limit, biological_power_limit) =
                genome.biological_joint_limits(joint_gene)?;
            let effective_inertia = genome.joint_effective_inertia(joint_gene)?;

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
                .motor_position(0.0, 0.0, 0.0)
                .motor_max_force(joint_gene.motor_max_torque)
                .build();

            let handle = impulse_joints.insert(parent, child, joint, true);
            motor_handles.push((
                handle,
                joint_gene.clone(),
                biological_torque_limit,
                biological_power_limit,
                effective_inertia,
            ));
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
        let max_actuator_power = motor_handles
            .iter()
            .map(|(_, _, _, power_limit, _)| *power_limit * config.motor_strength_multiplier)
            .sum::<f32>();
        let mut previous_mechanical_energy =
            Self::mechanical_energy(&handles, &rigid_bodies, gravity, config.dt)?;

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

            for (
                joint_handle,
                gene,
                biological_torque_limit,
                biological_power_limit,
                effective_inertia,
            ) in &motor_handles
            {
                let target = genome
                    .brain
                    .output_for_joint(gene.child_id)
                    .map(|expression| expression.evaluate(&context))
                    .unwrap_or_else(|| legacy_expression(gene).evaluate(&context))
                    .clamp(gene.limits_radians[0], gene.limits_radians[1]);

                let strength = config.motor_strength_multiplier;
                let mut effective_max_torque =
                    gene.motor_max_torque.min(*biological_torque_limit) * strength;

                if let Some(sensor) = context
                    .joints
                    .iter()
                    .find(|sensor| sensor.child_id == gene.child_id)
                {
                    let position_error = (target - sensor.angle_radians).abs();
                    let angular_speed = sensor.velocity_radians_per_second.abs();
                    let power_limit = *biological_power_limit * strength;
                    if angular_speed > 1.0e-4 && power_limit > 0.0 {
                        effective_max_torque =
                            effective_max_torque.min(power_limit / angular_speed);
                    }

                    let joint_span = (gene.limits_radians[1] - gene.limits_radians[0])
                        .abs()
                        .max(1.0e-3);
                    let characteristic_error = (joint_span * 0.5).max(1.0e-3);
                    let stiffness = effective_max_torque / characteristic_error;
                    let damping = 2.0 * (stiffness * *effective_inertia).sqrt();

                    let torque_proxy = (position_error * stiffness + angular_speed * damping)
                        .min(effective_max_torque);
                    motor_effort += torque_proxy * angular_speed * config.dt;

                    if let Some(joint) = impulse_joints.get_mut(*joint_handle, true) {
                        joint
                            .data
                            .set_motor_position(JointAxis::AngX, target, stiffness, damping);
                        joint
                            .data
                            .set_motor_max_force(JointAxis::AngX, effective_max_torque);
                    }
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

            for handle in handles.values() {
                let body = rigid_bodies
                    .get(*handle)
                    .ok_or_else(|| "creature body disappeared while checking speed".to_string())?;
                let speed = body.linvel().length();
                if !speed.is_finite() || speed > MAX_VALID_LINEAR_SPEED_M_S {
                    return Err(format!(
                        "unstable physics: body linear speed {:.3} m/s exceeds {:.1} m/s limit",
                        speed, MAX_VALID_LINEAR_SPEED_M_S
                    ));
                }
            }

            let mechanical_energy =
                Self::mechanical_energy(&handles, &rigid_bodies, gravity, config.dt)?;
            let numerical_tolerance_j = previous_mechanical_energy.abs() * 1.0e-4 + 1.0e-8;
            let maximum_explained_gain_j =
                max_actuator_power * config.dt * 2.0 + numerical_tolerance_j;
            if mechanical_energy - previous_mechanical_energy > maximum_explained_gain_j {
                return Err(format!(
                    "unstable physics: mechanical energy jumped from {:.3} J to {:.3} J in one step",
                    previous_mechanical_energy, mechanical_energy
                ));
            }
            previous_mechanical_energy = mechanical_energy;

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

    fn mechanical_energy(
        handles: &HashMap<u32, RigidBodyHandle>,
        rigid_bodies: &RigidBodySet,
        gravity: Vector,
        dt: f32,
    ) -> Result<f32, String> {
        let mut total = 0.0_f32;
        for handle in handles.values() {
            let body = rigid_bodies
                .get(*handle)
                .ok_or_else(|| "creature body disappeared while checking energy".to_string())?;
            total += body.kinetic_energy() + body.gravitational_potential_energy(dt, gravity);
        }
        if !total.is_finite() {
            return Err("unstable physics: non-finite mechanical energy".into());
        }
        Ok(total)
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
            let contact_tolerance_m =
                segment.half_extents.iter().copied().fold(0.0_f32, f32::max) * 0.02;
            let contact_tolerance_m = contact_tolerance_m.max(1.0e-6);
            let ground_contact = match world.surface_height_at(position.x, position.z) {
                Some(surface_y) if bottom_y <= surface_y + contact_tolerance_m => 1.0,
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

/// Resumable Rapier creature simulation used by live population visualization.
///
/// Unlike the full streaming runner, this keeps physics state alive between
/// calls so an entire population can advance one visual slice at a time
/// without precomputing complete trajectories or blocking worker threads.
pub struct CreatureVisualSession {
    config: SimulationConfig,
    genome: CreatureGenome,
    rigid_bodies: RigidBodySet,
    colliders: ColliderSet,
    handles: HashMap<u32, RigidBodyHandle>,
    root_body_handle: RigidBodyHandle,
    impulse_joints: ImpulseJointSet,
    motor_handles: Vec<(ImpulseJointHandle, JointGene, f32, f32, f32)>,
    gravity: Vector<Real>,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    multibody_joints: MultibodyJointSet,
    ccd_solver: CCDSolver,
    step: usize,
    total_steps: usize,
    motor_effort: f32,
    max_actuator_power: f32,
    previous_mechanical_energy: f32,
}

impl CreatureVisualSession {
    pub fn new(config: &SimulationConfig, genome: &CreatureGenome) -> Result<Self, String> {
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
                .linear_damping(0.0)
                .angular_damping(0.0)
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
        let mut motor_handles = Vec::new();

        for joint_gene in &genome.joints {
            let parent = *handles
                .get(&joint_gene.parent_id)
                .ok_or_else(|| "missing parent body handle".to_string())?;
            let child = *handles
                .get(&joint_gene.child_id)
                .ok_or_else(|| "missing child body handle".to_string())?;

            let axis = Vector::new(joint_gene.axis[0], joint_gene.axis[1], joint_gene.axis[2]);
            let (biological_torque_limit, biological_power_limit) =
                genome.biological_joint_limits(joint_gene)?;
            let effective_inertia = genome.joint_effective_inertia(joint_gene)?;

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
                .motor_position(0.0, 0.0, 0.0)
                .motor_max_force(joint_gene.motor_max_torque)
                .build();

            let handle = impulse_joints.insert(parent, child, joint, true);
            motor_handles.push((
                handle,
                joint_gene.clone(),
                biological_torque_limit,
                biological_power_limit,
                effective_inertia,
            ));
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
        let max_actuator_power = motor_handles
            .iter()
            .map(|(_, _, _, power_limit, _)| *power_limit * config.motor_strength_multiplier)
            .sum::<f32>();
        let previous_mechanical_energy =
            CreatureSimulator::mechanical_energy(&handles, &rigid_bodies, gravity, config.dt)?;

        Ok(Self {
            config: config.clone(),
            genome: genome.clone(),
            rigid_bodies,
            colliders,
            handles,
            root_body_handle,
            impulse_joints,
            motor_handles,
            gravity,
            integration_parameters,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            multibody_joints: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            step: 0,
            total_steps: config.step_count(),
            motor_effort: 0.0,
            max_actuator_power,
            previous_mechanical_energy,
        })
    }

    pub fn is_complete(&self) -> bool {
        self.step >= self.total_steps
    }

    pub fn snapshot(&self) -> Result<CreatureSnapshot, String> {
        CreatureSimulator::snapshot(
            &self.genome,
            &self.handles,
            &self.rigid_bodies,
            self.step,
            self.config.dt,
        )
    }

    pub fn advance_steps(&mut self, requested_steps: usize) -> Result<CreatureSnapshot, String> {
        let target_step = (self.step + requested_steps.max(1)).min(self.total_steps);
        while self.step < target_step {
            self.advance_one_step()?;
        }
        self.snapshot()
    }

    fn advance_one_step(&mut self) -> Result<(), String> {
        let next_step = self.step + 1;
        let time_seconds = self.step as f32 * self.config.dt;
        let context = CreatureSimulator::brain_context(
            &self.genome,
            &self.handles,
            &self.rigid_bodies,
            self.root_body_handle,
            time_seconds,
            &self.config.world,
        )?;

        for (
            joint_handle,
            gene,
            biological_torque_limit,
            biological_power_limit,
            effective_inertia,
        ) in &self.motor_handles
        {
            let target = self
                .genome
                .brain
                .output_for_joint(gene.child_id)
                .map(|expression| expression.evaluate(&context))
                .unwrap_or_else(|| legacy_expression(gene).evaluate(&context))
                .clamp(gene.limits_radians[0], gene.limits_radians[1]);

            let strength = self.config.motor_strength_multiplier;
            let mut effective_max_torque =
                gene.motor_max_torque.min(*biological_torque_limit) * strength;

            if let Some(sensor) = context
                .joints
                .iter()
                .find(|sensor| sensor.child_id == gene.child_id)
            {
                let position_error = (target - sensor.angle_radians).abs();
                let angular_speed = sensor.velocity_radians_per_second.abs();
                let power_limit = *biological_power_limit * strength;
                if angular_speed > 1.0e-4 && power_limit > 0.0 {
                    effective_max_torque = effective_max_torque.min(power_limit / angular_speed);
                }

                let joint_span = (gene.limits_radians[1] - gene.limits_radians[0])
                    .abs()
                    .max(1.0e-3);
                let characteristic_error = (joint_span * 0.5).max(1.0e-3);
                let stiffness = effective_max_torque / characteristic_error;
                let damping = 2.0 * (stiffness * *effective_inertia).sqrt();

                let torque_proxy = (position_error * stiffness + angular_speed * damping)
                    .min(effective_max_torque);
                self.motor_effort += torque_proxy * angular_speed * self.config.dt;

                if let Some(joint) = self.impulse_joints.get_mut(*joint_handle, true) {
                    joint
                        .data
                        .set_motor_position(JointAxis::AngX, target, stiffness, damping);
                    joint
                        .data
                        .set_motor_max_force(JointAxis::AngX, effective_max_torque);
                }
            }
        }

        self.physics_pipeline.step(
            self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_bodies,
            &mut self.colliders,
            &mut self.impulse_joints,
            &mut self.multibody_joints,
            &mut self.ccd_solver,
            &(),
            &(),
        );

        for handle in self.handles.values() {
            let body = self
                .rigid_bodies
                .get(*handle)
                .ok_or_else(|| "creature body disappeared while checking speed".to_string())?;
            let speed = body.linvel().length();
            if !speed.is_finite() || speed > MAX_VALID_LINEAR_SPEED_M_S {
                return Err(format!(
                    "unstable physics: body linear speed {:.3} m/s exceeds {:.1} m/s limit",
                    speed, MAX_VALID_LINEAR_SPEED_M_S
                ));
            }
        }

        let mechanical_energy = CreatureSimulator::mechanical_energy(
            &self.handles,
            &self.rigid_bodies,
            self.gravity,
            self.config.dt,
        )?;
        let numerical_tolerance_j = self.previous_mechanical_energy.abs() * 1.0e-4 + 1.0e-8;
        let maximum_explained_gain_j =
            self.max_actuator_power * self.config.dt * 2.0 + numerical_tolerance_j;
        if mechanical_energy - self.previous_mechanical_energy > maximum_explained_gain_j {
            return Err(format!(
                "unstable physics: mechanical energy jumped from {:.3} J to {:.3} J in one step",
                self.previous_mechanical_energy, mechanical_energy
            ));
        }
        self.previous_mechanical_energy = mechanical_energy;
        self.step = next_step;
        Ok(())
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
