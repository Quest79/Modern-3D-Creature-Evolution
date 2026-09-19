use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{CreatureGenome, JointGene, SegmentGene};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MutationConfig {
    pub min_segments: usize,
    pub max_segments: usize,
    pub min_half_extent: f32,
    pub max_half_extent: f32,
    pub structural_mutation_chance: f32,
}

impl Default for MutationConfig {
    fn default() -> Self {
        Self {
            min_segments: 2,
            max_segments: 12,
            min_half_extent: 0.10,
            max_half_extent: 0.85,
            structural_mutation_chance: 0.30,
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
    AddSegment,
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

    pub fn next_f32(&mut self) -> f32 {
        let value = (self.next_u64() >> 40) as u32;
        value as f32 / ((1_u32 << 24) - 1) as f32
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
    let mut mutations = Vec::with_capacity(mutation_count);

    for _ in 0..mutation_count {
        let structural = rng.chance(config.structural_mutation_chance);
        let record = if structural {
            mutate_structure(&mut genome, &mut rng, config)
                .or_else(|| mutate_numeric(&mut genome, &mut rng, config))
        } else {
            mutate_numeric(&mut genome, &mut rng, config)
                .or_else(|| mutate_structure(&mut genome, &mut rng, config))
        };

        if let Some(record) = record {
            mutations.push(record);
        }
    }

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
            half_extents: [0.50, 0.28, 0.34],
            initial_position: [0.0, 2.4, 0.0],
            density: 1.0,
            friction: 0.9,
        }],
        joints: Vec::new(),
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

    genome.validate()?;
    Ok(MutationResult { genome, mutations })
}

fn mutate_numeric(
    genome: &mut CreatureGenome,
    rng: &mut GenomeRng,
    config: &MutationConfig,
) -> Option<MutationRecord> {
    match rng.range_usize(5) {
        0 => resize_segment(genome, rng, config),
        1 => change_material(genome, rng),
        2 => change_motor(genome, rng),
        3 => change_joint_limits(genome, rng),
        _ => move_joint_anchor(genome, rng),
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
    let segment = genome.segments.get_mut(index)?;
    let old = segment.half_extents[axis];
    let factor = rng.range_f32(0.70, 1.35);
    segment.half_extents[axis] =
        (old * factor).clamp(config.min_half_extent, config.max_half_extent);

    Some(MutationRecord {
        kind: MutationKind::ResizeSegment,
        description: format!(
            "resized {} axis {} from {:.3} to {:.3}",
            segment.name, axis, old, segment.half_extents[axis]
        ),
    })
}

fn change_material(
    genome: &mut CreatureGenome,
    rng: &mut GenomeRng,
) -> Option<MutationRecord> {
    let index = rng.range_usize(genome.segments.len());
    let segment = genome.segments.get_mut(index)?;
    segment.density = (segment.density * rng.range_f32(0.80, 1.25)).clamp(0.25, 4.0);
    segment.friction = (segment.friction + rng.signed(0.25)).clamp(0.05, 2.5);

    Some(MutationRecord {
        kind: MutationKind::ChangeMaterial,
        description: format!(
            "changed {} density/friction to {:.2}/{:.2}",
            segment.name, segment.density, segment.friction
        ),
    })
}

fn change_motor(genome: &mut CreatureGenome, rng: &mut GenomeRng) -> Option<MutationRecord> {
    let index = rng.range_usize(genome.joints.len());
    let joint = genome.joints.get_mut(index)?;

    joint.motor_amplitude_radians =
        (joint.motor_amplitude_radians + rng.signed(0.25)).clamp(0.05, 1.4);
    joint.motor_frequency_hz =
        (joint.motor_frequency_hz * rng.range_f32(0.75, 1.30)).clamp(0.15, 4.0);
    joint.motor_phase_radians += rng.signed(0.5);
    joint.motor_max_torque =
        (joint.motor_max_torque * rng.range_f32(0.75, 1.35)).clamp(2.0, 80.0);

    Some(MutationRecord {
        kind: MutationKind::ChangeMotor,
        description: format!(
            "changed motor {}→{}: amplitude {:.2}, frequency {:.2} Hz, torque {:.1}",
            joint.parent_id,
            joint.child_id,
            joint.motor_amplitude_radians,
            joint.motor_frequency_hz,
            joint.motor_max_torque
        ),
    })
}

fn change_joint_limits(
    genome: &mut CreatureGenome,
    rng: &mut GenomeRng,
) -> Option<MutationRecord> {
    let index = rng.range_usize(genome.joints.len());
    let joint = genome.joints.get_mut(index)?;
    let center = (joint.limits_radians[0] + joint.limits_radians[1]) * 0.5 + rng.signed(0.12);
    let half_width = ((joint.limits_radians[1] - joint.limits_radians[0]) * 0.5
        * rng.range_f32(0.75, 1.25))
    .clamp(0.15, 1.45);

    joint.limits_radians = [
        (center - half_width).clamp(-1.55, 1.40),
        (center + half_width).clamp(-1.40, 1.55),
    ];

    if joint.limits_radians[1] - joint.limits_radians[0] < 0.20 {
        joint.limits_radians = [-0.25, 0.25];
    }

    Some(MutationRecord {
        kind: MutationKind::ChangeJointLimits,
        description: format!(
            "changed joint {}→{} limits to [{:.2}, {:.2}] rad",
            joint.parent_id,
            joint.child_id,
            joint.limits_radians[0],
            joint.limits_radians[1]
        ),
    })
}

fn move_joint_anchor(
    genome: &mut CreatureGenome,
    rng: &mut GenomeRng,
) -> Option<MutationRecord> {
    let index = rng.range_usize(genome.joints.len());
    let joint = genome.joints.get_mut(index)?;
    let axis = rng.range_usize(3);
    joint.parent_anchor[axis] += rng.signed(0.08);
    joint.child_anchor[axis] += rng.signed(0.08);

    Some(MutationRecord {
        kind: MutationKind::MoveJointAnchor,
        description: format!(
            "moved joint {}→{} attachment on axis {}",
            joint.parent_id, joint.child_id, axis
        ),
    })
}

fn add_segment(
    genome: &mut CreatureGenome,
    rng: &mut GenomeRng,
    config: &MutationConfig,
) -> Option<MutationRecord> {
    if genome.segments.len() >= config.max_segments {
        return None;
    }

    let parent_index = rng.range_usize(genome.segments.len());
    let parent = genome.segments.get(parent_index)?.clone();
    let id = genome
        .segments
        .iter()
        .map(|segment| segment.id)
        .max()
        .unwrap_or(0)
        + 1;

    let half_extents = [
        rng.range_f32(0.12, 0.40).clamp(config.min_half_extent, config.max_half_extent),
        rng.range_f32(0.18, 0.62).clamp(config.min_half_extent, config.max_half_extent),
        rng.range_f32(0.12, 0.36).clamp(config.min_half_extent, config.max_half_extent),
    ];

    let direction_index = rng.range_usize(6);
    let mut direction = [0.0_f32; 3];
    let axis_index = direction_index / 2;
    let sign = if direction_index % 2 == 0 { -1.0 } else { 1.0 };
    direction[axis_index] = sign;

    let mut initial_position = parent.initial_position;
    initial_position[axis_index] +=
        sign * (parent.half_extents[axis_index] + half_extents[axis_index]);

    // Lift generated descendants slightly so they do not begin buried in the
    // floor. Physics will settle the morphology immediately.
    initial_position[1] = initial_position[1].max(0.35 + half_extents[1]);

    let child = SegmentGene {
        id,
        name: format!("segment_{id}"),
        half_extents,
        initial_position,
        density: rng.range_f32(0.7, 1.4),
        friction: rng.range_f32(0.55, 1.45),
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
    genome.joints.push(JointGene {
        parent_id: parent.id,
        child_id: id,
        parent_anchor,
        child_anchor,
        axis: joint_axis,
        limits_radians: [-0.90, 0.90],
        motor_amplitude_radians: rng.range_f32(0.25, 0.85),
        motor_frequency_hz: rng.range_f32(0.45, 2.0),
        motor_phase_radians: rng.range_f32(0.0, std::f32::consts::TAU),
        motor_stiffness: rng.range_f32(18.0, 42.0),
        motor_damping: rng.range_f32(2.5, 7.0),
        motor_max_torque: rng.range_f32(8.0, 30.0),
    });

    Some(MutationRecord {
        kind: MutationKind::AddSegment,
        description: format!("added segment {id} to parent {}", parent.id),
    })
}

fn remove_leaf_segment(
    genome: &mut CreatureGenome,
    rng: &mut GenomeRng,
) -> Option<MutationRecord> {
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
    fn many_mutations_keep_genome_valid() {
        let source = CreatureGenome::three_segment_walker();
        let result =
            mutate_genome(&source, 987_654_321, 250, &MutationConfig::default()).unwrap();
        assert!(result.genome.validate().is_ok());
    }
}
