use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{
    BIOLOGICAL_MAX_CONTACT_FRICTION, BiologicalMaterial, BrainGenome, CreatureGenome, Expression,
    JointGene, SegmentGene, SensorKind,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MutationConfig {
    pub min_segments: usize,
    pub max_segments: usize,
    pub min_half_extent: f32,
    pub max_half_extent: f32,
    pub structural_mutation_chance: f32,
    /// Fraction of structural mutations that become a larger morphology change
    /// such as adding a multi-segment limb.
    #[serde(default = "default_major_structural_mutation_chance")]
    pub major_structural_mutation_chance: f32,
}

fn default_major_structural_mutation_chance() -> f32 {
    0.10
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            min_segments: 2,
            max_segments: 12,
            // Broad articulated-animal scale: 0.2 mm to 30 m full segment
            // dimension. This is a solver/world-scale bound, not an anatomy bound.
            min_half_extent: 0.0001,
            max_half_extent: 15.0,
            structural_mutation_chance: 0.30,
            major_structural_mutation_chance: default_major_structural_mutation_chance(),
        }
    }
}

impl MutationConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.min_segments == 0 {
            return Err("min_segments must be at least 1".into());
        }
        if self.max_segments < self.min_segments {
            return Err("max_segments must be greater than or equal to min_segments".into());
        }
        if !self.min_half_extent.is_finite()
            || !self.max_half_extent.is_finite()
            || self.min_half_extent <= 0.0
            || self.max_half_extent < self.min_half_extent
        {
            return Err("invalid mutation half-extent range".into());
        }
        if !self.structural_mutation_chance.is_finite()
            || !(0.0..=1.0).contains(&self.structural_mutation_chance)
        {
            return Err("structural_mutation_chance must be between 0 and 1".into());
        }
        if !self.major_structural_mutation_chance.is_finite()
            || !(0.0..=1.0).contains(&self.major_structural_mutation_chance)
        {
            return Err("major_structural_mutation_chance must be between 0 and 1".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MutationKind {
    ResizeSegment,
    ChangeMaterial,
    ChangeMotor,
    ChangeJointLimits,
    MoveJointAnchor,
    ChangeBrainConstant,
    ReplaceBrainExpression,
    ReplaceBrainSubtree,
    WrapBrainExpression,
    WrapBrainSubtree,
    AddSegment,
    AddLimb,
    RemoveLeafSegment,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MutationRecord {
    pub kind: MutationKind,
    pub description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MutationResult {
    pub genome: CreatureGenome,
    pub mutations: Vec<MutationRecord>,
}

/// Tiny deterministic RNG so experiments can reproduce mutations exactly
/// without depending on platform/global random state.
#[derive(Clone, Debug)]
pub struct GenomeRng {
    state: u64,
}

impl GenomeRng {
    pub fn new(seed: u64) -> Self {
        let state = if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        };
        Self { state }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn state(&self) -> u64 {
        self.state
    }

    pub fn from_state(state: u64) -> Self {
        Self {
            state: if state == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                state
            },
        }
    }

    pub fn next_f32(&mut self) -> f32 {
        let value = (self.next_u64() >> 40) as u32;
        value as f32 / ((1_u32 << 24) - 1) as f32
    }

    pub fn next_seed(&mut self) -> u64 {
        self.next_u64()
    }

    pub fn range_f32(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }

    pub fn range_usize(&mut self, end_exclusive: usize) -> usize {
        if end_exclusive <= 1 {
            0
        } else {
            (self.next_u64() as usize) % end_exclusive
        }
    }

    pub fn chance(&mut self, probability: f32) -> bool {
        self.next_f32() < probability.clamp(0.0, 1.0)
    }

    pub fn signed(&mut self, magnitude: f32) -> f32 {
        self.range_f32(-magnitude, magnitude)
    }
}

pub fn mutate_genome(
    source: &CreatureGenome,
    seed: u64,
    mutation_count: usize,
    config: &MutationConfig,
) -> Result<MutationResult, String> {
    config.validate()?;
    source.validate()?;

    let mut rng = GenomeRng::new(seed);
    let mut genome = source.clone();
    genome
        .brain
        .sync_with_structure(&genome.joints, &genome.segments);
    let mut mutations = Vec::with_capacity(mutation_count);

    for _ in 0..mutation_count {
        let structural = rng.chance(config.structural_mutation_chance);
        let record = if structural {
            let major = rng.chance(config.major_structural_mutation_chance);
            if major {
                add_limb(&mut genome, &mut rng, config)
                    .or_else(|| mutate_structure(&mut genome, &mut rng, config))
                    .or_else(|| mutate_numeric(&mut genome, &mut rng, config))
            } else {
                mutate_structure(&mut genome, &mut rng, config)
                    .or_else(|| mutate_numeric(&mut genome, &mut rng, config))
            }
        } else {
            mutate_numeric(&mut genome, &mut rng, config)
                .or_else(|| mutate_structure(&mut genome, &mut rng, config))
        };

        if let Some(record) = record {
            mutations.push(record);
        }
    }

    genome
        .brain
        .sync_with_structure(&genome.joints, &genome.segments);
    genome.name = format!("{} • mutated {}", source.name, seed);
    genome.validate()?;

    Ok(MutationResult { genome, mutations })
}

pub fn random_creature(
    seed: u64,
    target_segments: usize,
    config: &MutationConfig,
) -> Result<MutationResult, String> {
    config.validate()?;

    let target_segments = target_segments.clamp(config.min_segments, config.max_segments);
    let mut genome = CreatureGenome {
        name: format!("Random Creature {seed}"),
        segments: vec![SegmentGene {
            id: 0,
            name: "root".to_string(),
            half_extents: [0.25, 0.14, 0.17],
            initial_position: [0.0, 0.80, 0.0],
            material: BiologicalMaterial::SoftTissue,
            density: BiologicalMaterial::SoftTissue.representative_density_kg_m3(),
            friction: 0.6,
        }],
        joints: Vec::new(),
        brain: BrainGenome::default(),
    };

    let mut rng = GenomeRng::new(seed);
    let mut mutations = Vec::new();

    while genome.segments.len() < target_segments {
        if let Some(record) = add_segment(&mut genome, &mut rng, config) {
            mutations.push(record);
        } else {
            break;
        }
    }

    // Give every generated body a few non-structural variations so identical
    // topologies still differ in proportions and motor behavior.
    let numeric_passes = target_segments.saturating_mul(3);
    for _ in 0..numeric_passes {
        if let Some(record) = mutate_numeric(&mut genome, &mut rng, config) {
            mutations.push(record);
        }
    }

    genome
        .brain
        .sync_with_structure(&genome.joints, &genome.segments);
    genome.validate()?;
    Ok(MutationResult { genome, mutations })
}

fn mutate_numeric(
    genome: &mut CreatureGenome,
    rng: &mut GenomeRng,
    config: &MutationConfig,
) -> Option<MutationRecord> {
    match rng.range_usize(7) {
        0 => resize_segment(genome, rng, config),
        1 => change_material(genome, rng),
        2 => change_motor(genome, rng),
        3 => change_joint_limits(genome, rng),
        4 => move_joint_anchor(genome, rng),
        _ => mutate_brain(genome, rng),
    }
}

fn mutate_structure(
    genome: &mut CreatureGenome,
    rng: &mut GenomeRng,
    config: &MutationConfig,
) -> Option<MutationRecord> {
    let can_add = genome.segments.len() < config.max_segments;
    let can_remove = genome.segments.len() > config.min_segments;

    match (can_add, can_remove) {
        (true, true) if rng.chance(0.62) => add_segment(genome, rng, config),
        (true, false) => add_segment(genome, rng, config),
        (_, true) => remove_leaf_segment(genome, rng),
        _ => None,
    }
}

fn resize_segment(
    genome: &mut CreatureGenome,
    rng: &mut GenomeRng,
    config: &MutationConfig,
) -> Option<MutationRecord> {
    let index = rng.range_usize(genome.segments.len());
    let axis = rng.range_usize(3);
    let segment_id = genome.segments.get(index)?.id;
    let anchor_floor = genome
        .joints
        .iter()
        .filter_map(|joint| {
            if joint.parent_id == segment_id {
                Some(joint.parent_anchor[axis].abs())
            } else if joint.child_id == segment_id {
                Some(joint.child_anchor[axis].abs())
            } else {
                None
            }
        })
        .fold(config.min_half_extent, f32::max);

    let segment = genome.segments.get_mut(index)?;
    let old = segment.half_extents[axis];
    let factor = rng.range_f32(0.70, 1.35);
    segment.half_extents[axis] = (old * factor).clamp(anchor_floor, config.max_half_extent);

    Some(MutationRecord {
        kind: MutationKind::ResizeSegment,
        description: format!(
            "resized {} axis {} from {:.3} to {:.3}",
            segment.name, axis, old, segment.half_extents[axis]
        ),
    })
}

fn change_material(genome: &mut CreatureGenome, rng: &mut GenomeRng) -> Option<MutationRecord> {
    let index = rng.range_usize(genome.segments.len());
    let segment = genome.segments.get_mut(index)?;

    if rng.chance(0.35) {
        segment.material = random_biological_material(rng);
        let (min_density, max_density) = segment.material.density_range_kg_m3();
        segment.density = rng.range_f32(min_density, max_density);
    } else {
        let (min_density, max_density) = segment.material.density_range_kg_m3();
        segment.density =
            (segment.density * rng.range_f32(0.80, 1.25)).clamp(min_density, max_density);
    }
    segment.friction =
        (segment.friction + rng.signed(0.20)).clamp(0.0, BIOLOGICAL_MAX_CONTACT_FRICTION);

    Some(MutationRecord {
        kind: MutationKind::ChangeMaterial,
        description: format!(
            "changed {} material to {:?}, density {:.1} kg/m^3, friction {:.2}",
            segment.name, segment.material, segment.density, segment.friction
        ),
    })
}

fn random_biological_material(rng: &mut GenomeRng) -> BiologicalMaterial {
    match rng.range_usize(9) {
        0 => BiologicalMaterial::PorousPlant,
        1 => BiologicalMaterial::DensePlant,
        2 => BiologicalMaterial::Adipose,
        3 => BiologicalMaterial::SoftTissue,
        4 => BiologicalMaterial::FibrousTissue,
        5 => BiologicalMaterial::TrabecularBone,
        6 => BiologicalMaterial::CorticalBone,
        7 => BiologicalMaterial::Exoskeleton,
        _ => BiologicalMaterial::MineralizedTissue,
    }
}

fn change_motor(genome: &mut CreatureGenome, rng: &mut GenomeRng) -> Option<MutationRecord> {
    let index = rng.range_usize(genome.joints.len());
    let biological_torque_limit = {
        let joint = genome.joints.get(index)?;
        genome.biological_joint_limits(joint).ok()?.0
    };
    let joint = genome.joints.get_mut(index)?;

    // Controller gains are derived from physical inertia at runtime. Evolution
    // changes only biological actuation traits: requested torque, excursion, and
    // contraction-cycle frequency.
    joint.motor_max_torque =
        (joint.motor_max_torque * rng.range_f32(0.70, 1.40)).clamp(0.0, biological_torque_limit);

    let half_span = ((joint.limits_radians[1] - joint.limits_radians[0]) * 0.5).max(0.0);
    joint.motor_amplitude_radians =
        (joint.motor_amplitude_radians * rng.range_f32(0.75, 1.25)).clamp(0.0, half_span);

    joint.motor_frequency_hz =
        (joint.motor_frequency_hz * rng.range_f32(0.70, 1.40)).clamp(0.05, 20.0);

    Some(MutationRecord {
        kind: MutationKind::ChangeMotor,
        description: format!(
            "changed actuator {}→{}: torque {:.2} N·m / {:.2} N·m ceiling, amplitude {:.3} rad, frequency {:.3} Hz",
            joint.parent_id,
            joint.child_id,
            joint.motor_max_torque,
            biological_torque_limit,
            joint.motor_amplitude_radians,
            joint.motor_frequency_hz
        ),
    })
}

fn change_joint_limits(genome: &mut CreatureGenome, rng: &mut GenomeRng) -> Option<MutationRecord> {
    let index = rng.range_usize(genome.joints.len());
    let joint = genome.joints.get_mut(index)?;

    // Keep a tiny floating-point guard below the exact 180-degree validation
    // ceiling. Using exactly PI/2 here can round (max - min) a few ulps above PI.
    const JOINT_SPAN_GUARD_RADIANS: f32 = 1.0e-5;
    let max_half_width = std::f32::consts::FRAC_PI_2 - JOINT_SPAN_GUARD_RADIANS;
    let half_width =
        ((joint.limits_radians[1] - joint.limits_radians[0]) * 0.5 * rng.range_f32(0.70, 1.30))
            .clamp(0.005, max_half_width);
    let center = ((joint.limits_radians[0] + joint.limits_radians[1]) * 0.5 + rng.signed(0.12))
        .clamp(
            -std::f32::consts::PI + half_width,
            std::f32::consts::PI - half_width,
        );

    joint.limits_radians = [center - half_width, center + half_width];

    Some(MutationRecord {
        kind: MutationKind::ChangeJointLimits,
        description: format!(
            "changed joint {}→{} limits to [{:.2}, {:.2}] rad",
            joint.parent_id, joint.child_id, joint.limits_radians[0], joint.limits_radians[1]
        ),
    })
}

fn move_joint_anchor(genome: &mut CreatureGenome, rng: &mut GenomeRng) -> Option<MutationRecord> {
    let index = rng.range_usize(genome.joints.len());
    let (parent_id, child_id, parent_anchor, child_anchor) = {
        let joint = genome.joints.get(index)?;
        (
            joint.parent_id,
            joint.child_id,
            joint.parent_anchor,
            joint.child_anchor,
        )
    };
    let parent = genome
        .segments
        .iter()
        .find(|segment| segment.id == parent_id)?
        .clone();
    let child = genome
        .segments
        .iter()
        .find(|segment| segment.id == child_id)?
        .clone();

    // Identify the attachment face and move only along that face. Applying the
    // exact same local delta to both anchors keeps their world positions
    // coincident at spawn, eliminating solver-created launch energy.
    let normal_axis = (0..3)
        .max_by(|&a, &b| {
            let a_ratio = parent_anchor[a].abs() / parent.half_extents[a].max(1.0e-6);
            let b_ratio = parent_anchor[b].abs() / parent.half_extents[b].max(1.0e-6);
            a_ratio.total_cmp(&b_ratio)
        })
        .unwrap_or(0);
    let tangent_axes: Vec<usize> = (0..3).filter(|axis| *axis != normal_axis).collect();
    let axis = tangent_axes[rng.range_usize(tangent_axes.len())];

    let maximum_step = parent.half_extents[axis].min(child.half_extents[axis]) * 0.10;
    let requested_delta = rng.signed(maximum_step);
    let minimum_delta = (-parent.half_extents[axis] - parent_anchor[axis])
        .max(-child.half_extents[axis] - child_anchor[axis]);
    let maximum_delta = (parent.half_extents[axis] - parent_anchor[axis])
        .min(child.half_extents[axis] - child_anchor[axis]);
    let delta = requested_delta.clamp(minimum_delta, maximum_delta);

    let joint = genome.joints.get_mut(index)?;
    joint.parent_anchor[axis] += delta;
    joint.child_anchor[axis] += delta;

    Some(MutationRecord {
        kind: MutationKind::MoveJointAnchor,
        description: format!(
            "moved joint {}→{} attachment {:.4} m along local axis {}",
            joint.parent_id, joint.child_id, delta, axis
        ),
    })
}

fn mutate_brain(genome: &mut CreatureGenome, rng: &mut GenomeRng) -> Option<MutationRecord> {
    genome
        .brain
        .sync_with_structure(&genome.joints, &genome.segments);
    if genome.brain.outputs.is_empty() {
        return None;
    }

    let joint_ids: Vec<u32> = genome.joints.iter().map(|joint| joint.child_id).collect();
    let segment_ids: Vec<u32> = genome.segments.iter().map(|segment| segment.id).collect();
    let index = rng.range_usize(genome.brain.outputs.len());
    let output = genome.brain.outputs.get_mut(index)?;
    let child_id = output.joint_child_id;

    match rng.range_usize(4) {
        0 => {
            if perturb_one_constant(&mut output.expression, rng) {
                Some(MutationRecord {
                    kind: MutationKind::ChangeBrainConstant,
                    description: format!("changed brain constant for joint child {child_id}"),
                })
            } else {
                let replacement = random_expression(rng, 2, &joint_ids, &segment_ids);
                let target = rng.range_usize(output.expression.node_count());
                output.expression.replace_subtree(target, replacement);
                Some(MutationRecord {
                    kind: MutationKind::ReplaceBrainSubtree,
                    description: format!(
                        "replaced brain subtree {target} for joint child {child_id}"
                    ),
                })
            }
        }
        1 => {
            let target = rng.range_usize(output.expression.node_count());
            let replacement = random_expression(rng, 2, &joint_ids, &segment_ids);
            let original = output.expression.clone();
            output.expression.replace_subtree(target, replacement);

            if output.expression.validate().is_err() {
                output.expression = original;
                return None;
            }

            Some(MutationRecord {
                kind: MutationKind::ReplaceBrainSubtree,
                description: format!("replaced brain subtree {target} for joint child {child_id}"),
            })
        }
        2 => {
            let target = rng.range_usize(output.expression.node_count());
            let selected = output.expression.subtree_clone(target)?;
            let wrapped = match rng.range_usize(5) {
                0 => Expression::Sin(Box::new(selected)),
                1 => Expression::Cos(Box::new(selected)),
                2 => Expression::Negate(Box::new(selected)),
                3 => Expression::Add(
                    Box::new(selected),
                    Box::new(Expression::Constant(rng.signed(0.35))),
                ),
                _ => Expression::Multiply(
                    Box::new(selected),
                    Box::new(Expression::Constant(rng.range_f32(0.65, 1.35))),
                ),
            };

            let original = output.expression.clone();
            output.expression.replace_subtree(target, wrapped);
            if output.expression.validate().is_err() {
                output.expression = original;
                return None;
            }

            Some(MutationRecord {
                kind: MutationKind::WrapBrainSubtree,
                description: format!("wrapped brain subtree {target} for joint child {child_id}"),
            })
        }
        _ => {
            output.expression = random_expression(rng, 3, &joint_ids, &segment_ids);
            Some(MutationRecord {
                kind: MutationKind::ReplaceBrainExpression,
                description: format!("replaced brain expression for joint child {child_id}"),
            })
        }
    }
}

pub fn crossover_brain_subtree(
    primary: &CreatureGenome,
    donor: &CreatureGenome,
    rng: &mut GenomeRng,
) -> CreatureGenome {
    let mut child = primary.clone();
    child.brain.sync_with_joints(&child.joints);

    let compatible_outputs: Vec<usize> = child
        .brain
        .outputs
        .iter()
        .enumerate()
        .filter_map(|(index, output)| {
            donor
                .brain
                .output_for_joint(output.joint_child_id)
                .map(|_| index)
        })
        .collect();

    if compatible_outputs.is_empty() {
        return child;
    }

    let output_index = compatible_outputs[rng.range_usize(compatible_outputs.len())];
    let child_id = child.brain.outputs[output_index].joint_child_id;
    let Some(donor_expression) = donor.brain.output_for_joint(child_id) else {
        return child;
    };

    let donor_subtree_index = rng.range_usize(donor_expression.node_count());
    let Some(donor_subtree) = donor_expression.subtree_clone(donor_subtree_index) else {
        return child;
    };

    let recipient_expression = &mut child.brain.outputs[output_index].expression;
    let recipient_subtree_index = rng.range_usize(recipient_expression.node_count());
    let original = recipient_expression.clone();
    recipient_expression.replace_subtree(recipient_subtree_index, donor_subtree);

    if recipient_expression.validate().is_err() || child.validate().is_err() {
        child.brain.outputs[output_index].expression = original;
    }

    child
}

fn perturb_one_constant(expression: &mut Expression, rng: &mut GenomeRng) -> bool {
    match expression {
        Expression::Constant(value) => {
            *value = (*value + rng.signed(0.35)).clamp(-12.0, 12.0);
            true
        }
        Expression::Add(left, right)
        | Expression::Subtract(left, right)
        | Expression::Multiply(left, right) => {
            let (first, second) = if rng.chance(0.5) {
                (left, right)
            } else {
                (right, left)
            };
            perturb_one_constant(first, rng) || perturb_one_constant(second, rng)
        }
        Expression::Negate(value)
        | Expression::Sin(value)
        | Expression::Cos(value)
        | Expression::Clamp { value, .. } => perturb_one_constant(value, rng),
        Expression::Sensor(_) => false,
    }
}

fn random_expression(
    rng: &mut GenomeRng,
    depth: usize,
    joint_ids: &[u32],
    segment_ids: &[u32],
) -> Expression {
    if depth == 0 || rng.chance(0.30) {
        return random_terminal(rng, joint_ids, segment_ids);
    }

    match rng.range_usize(8) {
        0 => Expression::Add(
            Box::new(random_expression(rng, depth - 1, joint_ids, segment_ids)),
            Box::new(random_expression(rng, depth - 1, joint_ids, segment_ids)),
        ),
        1 => Expression::Subtract(
            Box::new(random_expression(rng, depth - 1, joint_ids, segment_ids)),
            Box::new(random_expression(rng, depth - 1, joint_ids, segment_ids)),
        ),
        2 => Expression::Multiply(
            Box::new(random_expression(rng, depth - 1, joint_ids, segment_ids)),
            Box::new(random_expression(rng, depth - 1, joint_ids, segment_ids)),
        ),
        3 => Expression::Negate(Box::new(random_expression(
            rng,
            depth - 1,
            joint_ids,
            segment_ids,
        ))),
        4 => Expression::Sin(Box::new(random_expression(
            rng,
            depth - 1,
            joint_ids,
            segment_ids,
        ))),
        5 => Expression::Cos(Box::new(random_expression(
            rng,
            depth - 1,
            joint_ids,
            segment_ids,
        ))),
        6 => Expression::Clamp {
            value: Box::new(random_expression(rng, depth - 1, joint_ids, segment_ids)),
            min: -1.25,
            max: 1.25,
        },
        _ => random_terminal(rng, joint_ids, segment_ids),
    }
}

fn random_terminal(rng: &mut GenomeRng, joint_ids: &[u32], segment_ids: &[u32]) -> Expression {
    if rng.chance(0.35) {
        Expression::Constant(rng.range_f32(-1.5, 1.5))
    } else {
        Expression::Sensor(random_sensor(rng, joint_ids, segment_ids))
    }
}

fn random_sensor(rng: &mut GenomeRng, joint_ids: &[u32], segment_ids: &[u32]) -> SensorKind {
    match rng.range_usize(15) {
        0 => SensorKind::Time,
        1 => SensorKind::RootHeight,
        2 => SensorKind::RootVelocityX,
        3 => SensorKind::RootVelocityY,
        4 => SensorKind::RootVelocityZ,
        5 => SensorKind::RootAngularVelocityX,
        6 => SensorKind::RootAngularVelocityY,
        7 => SensorKind::RootAngularVelocityZ,
        8 => SensorKind::RootRotationX,
        9 => SensorKind::RootRotationY,
        10 => SensorKind::RootRotationZ,
        11 => SensorKind::RootRotationW,
        12 if !joint_ids.is_empty() => {
            SensorKind::JointAngle(joint_ids[rng.range_usize(joint_ids.len())])
        }
        13 if !joint_ids.is_empty() => {
            SensorKind::JointVelocity(joint_ids[rng.range_usize(joint_ids.len())])
        }
        14 if !segment_ids.is_empty() => {
            SensorKind::SegmentGroundContact(segment_ids[rng.range_usize(segment_ids.len())])
        }
        _ => SensorKind::Time,
    }
}

fn add_segment(
    genome: &mut CreatureGenome,
    rng: &mut GenomeRng,
    config: &MutationConfig,
) -> Option<MutationRecord> {
    if genome.segments.len() >= config.max_segments {
        return None;
    }
    let parent_id = genome
        .segments
        .get(rng.range_usize(genome.segments.len()))?
        .id;
    add_segment_to_parent(genome, rng, config, parent_id).map(|(record, _)| record)
}

fn add_limb(
    genome: &mut CreatureGenome,
    rng: &mut GenomeRng,
    config: &MutationConfig,
) -> Option<MutationRecord> {
    let available = config.max_segments.saturating_sub(genome.segments.len());
    if available < 2 || genome.segments.is_empty() {
        return None;
    }

    let original = genome.clone();
    let root_parent_id = genome
        .segments
        .get(rng.range_usize(genome.segments.len()))?
        .id;
    let max_limb_segments = available.min(4);
    let limb_segments = 2 + rng.range_usize(max_limb_segments - 1);
    let mut parent_id = root_parent_id;
    let mut added_ids = Vec::with_capacity(limb_segments);

    for _ in 0..limb_segments {
        let Some((_record, child_id)) =
            add_segment_to_parent(genome, rng, config, parent_id)
        else {
            *genome = original;
            return None;
        };
        added_ids.push(child_id);
        parent_id = child_id;
    }

    Some(MutationRecord {
        kind: MutationKind::AddLimb,
        description: format!(
            "major morphology mutation added {}-segment limb {:?} to parent {}",
            added_ids.len(),
            added_ids,
            root_parent_id
        ),
    })
}

fn add_segment_to_parent(
    genome: &mut CreatureGenome,
    rng: &mut GenomeRng,
    config: &MutationConfig,
    parent_id: u32,
) -> Option<(MutationRecord, u32)> {
    if genome.segments.len() >= config.max_segments {
        return None;
    }

    let parent = genome
        .segments
        .iter()
        .find(|segment| segment.id == parent_id)?
        .clone();
    let id = genome
        .segments
        .iter()
        .map(|segment| segment.id)
        .max()
        .unwrap_or(0)
        + 1;

    let half_extents = [
        (parent.half_extents[0] * rng.range_f32(0.45, 1.20))
            .clamp(config.min_half_extent, config.max_half_extent),
        (parent.half_extents[1] * rng.range_f32(0.45, 1.20))
            .clamp(config.min_half_extent, config.max_half_extent),
        (parent.half_extents[2] * rng.range_f32(0.45, 1.20))
            .clamp(config.min_half_extent, config.max_half_extent),
    ];

    let direction_index = rng.range_usize(6);
    let axis_index = direction_index / 2;
    let sign = if direction_index.is_multiple_of(2) {
        -1.0
    } else {
        1.0
    };

    let mut initial_position = parent.initial_position;
    initial_position[axis_index] +=
        sign * (parent.half_extents[axis_index] + half_extents[axis_index]);

    let material = random_biological_material(rng);
    let (min_density, max_density) = material.density_range_kg_m3();
    let child = SegmentGene {
        id,
        name: format!("segment_{id}"),
        half_extents,
        initial_position,
        material,
        density: rng.range_f32(min_density, max_density),
        friction: rng.range_f32(0.10, BIOLOGICAL_MAX_CONTACT_FRICTION),
    };

    let mut parent_anchor = [0.0_f32; 3];
    let mut child_anchor = [0.0_f32; 3];
    parent_anchor[axis_index] = sign * parent.half_extents[axis_index];
    child_anchor[axis_index] = -sign * half_extents[axis_index];

    let joint_axis = match axis_index {
        0 => [0.0, 0.0, 1.0],
        1 => [0.0, 0.0, 1.0],
        _ => [1.0, 0.0, 0.0],
    };

    genome.segments.push(child);
    let characteristic_length_m =
        (half_extents[0].max(half_extents[1]).max(half_extents[2]) * 2.0).max(0.0002);
    let gravity_scale_hz = (9.81 / characteristic_length_m).sqrt() / std::f32::consts::TAU;
    let seed_frequency_hz = (gravity_scale_hz * rng.range_f32(0.5, 2.0)).clamp(0.05, 20.0);

    let mut joint = JointGene {
        parent_id: parent.id,
        child_id: id,
        parent_anchor,
        child_anchor,
        axis: joint_axis,
        limits_radians: [-0.90, 0.90],
        motor_amplitude_radians: rng.range_f32(0.15, 0.85),
        motor_frequency_hz: seed_frequency_hz,
        motor_phase_radians: rng.range_f32(0.0, std::f32::consts::TAU),
        motor_stiffness: 0.0,
        motor_damping: 0.0,
        motor_max_torque: 0.0,
    };
    let biological_torque_limit = genome.biological_joint_limits(&joint).ok()?.0;
    joint.motor_max_torque = biological_torque_limit * rng.range_f32(0.05, 0.15);
    genome.joints.push(joint);

    lift_genome_above_floor(genome, 0.001);
    genome
        .brain
        .sync_with_structure(&genome.joints, &genome.segments);

    Some((
        MutationRecord {
            kind: MutationKind::AddSegment,
            description: format!("added segment {id} to parent {}", parent.id),
        },
        id,
    ))
}

fn lift_genome_above_floor(genome: &mut CreatureGenome, clearance_m: f32) {
    let minimum_bottom = genome
        .segments
        .iter()
        .map(|segment| segment.initial_position[1] - segment.half_extents[1])
        .fold(f32::INFINITY, f32::min);
    if !minimum_bottom.is_finite() {
        return;
    }

    let lift = (clearance_m - minimum_bottom).max(0.0);
    if lift > 0.0 {
        for segment in &mut genome.segments {
            segment.initial_position[1] += lift;
        }
    }
}

fn remove_leaf_segment(genome: &mut CreatureGenome, rng: &mut GenomeRng) -> Option<MutationRecord> {
    if genome.segments.len() <= 1 {
        return None;
    }

    let parents: HashSet<u32> = genome.joints.iter().map(|joint| joint.parent_id).collect();
    let root_id = genome.segments.first()?.id;
    let leaves: Vec<u32> = genome
        .segments
        .iter()
        .map(|segment| segment.id)
        .filter(|id| *id != root_id && !parents.contains(id))
        .collect();

    if leaves.is_empty() {
        return None;
    }

    let id = leaves[rng.range_usize(leaves.len())];
    genome.segments.retain(|segment| segment.id != id);
    genome
        .joints
        .retain(|joint| joint.parent_id != id && joint.child_id != id);
    genome
        .brain
        .sync_with_structure(&genome.joints, &genome.segments);

    Some(MutationRecord {
        kind: MutationKind::RemoveLeafSegment,
        description: format!("removed leaf segment {id}"),
    })
}

#[cfg(test)]
mod tests {
    use super::{MutationConfig, mutate_genome, random_creature};
    use crate::CreatureGenome;

    #[test]
    fn mutation_is_reproducible_for_same_seed() {
        let source = CreatureGenome::three_segment_walker();
        let config = MutationConfig::default();
        let a = mutate_genome(&source, 1234, 20, &config).unwrap();
        let b = mutate_genome(&source, 1234, 20, &config).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn random_creature_respects_segment_limit() {
        let config = MutationConfig {
            max_segments: 6,
            ..MutationConfig::default()
        };
        let result = random_creature(42, 6, &config).unwrap();
        assert_eq!(result.genome.segments.len(), 6);
        assert!(result.genome.validate().is_ok());
    }

    #[test]
    fn major_structural_mutation_can_add_a_multi_segment_limb() {
        let mut genome = CreatureGenome::three_segment_walker();
        let mut rng = super::GenomeRng::new(12345);
        let config = MutationConfig {
            max_segments: 12,
            ..MutationConfig::default()
        };
        let before = genome.segments.len();
        let record = super::add_limb(&mut genome, &mut rng, &config).unwrap();
        assert_eq!(record.kind, super::MutationKind::AddLimb);
        assert!(genome.segments.len() >= before + 2);
        assert!(genome.validate().is_ok());
    }

    #[test]
    fn many_mutations_keep_genome_valid() {
        let source = CreatureGenome::three_segment_walker();
        let result = mutate_genome(&source, 987_654_321, 250, &MutationConfig::default()).unwrap();
        assert!(result.genome.validate().is_ok());
    }

    #[test]
    fn joint_limit_mutations_never_exceed_validation_span() {
        let source = CreatureGenome::three_segment_walker();
        let config = MutationConfig {
            structural_mutation_chance: 0.0,
            ..MutationConfig::default()
        };

        for seed in 0..1_000_u64 {
            if let Ok(result) = mutate_genome(&source, seed, 100, &config) {
                for joint in &result.genome.joints {
                    assert!(
                        joint.limits_radians[1] - joint.limits_radians[0] <= std::f32::consts::PI
                    );
                }
            }
        }
    }
}
