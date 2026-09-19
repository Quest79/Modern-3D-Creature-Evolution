use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{JointGene, SegmentGene};

const MAX_EXPRESSION_DEPTH: usize = 16;
const VALUE_LIMIT: f32 = 1000.0;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SensorKind {
    Time,
    RootHeight,
    RootVelocityX,
    RootVelocityY,
    RootVelocityZ,
    RootAngularVelocityX,
    RootAngularVelocityY,
    RootAngularVelocityZ,
    RootRotationX,
    RootRotationY,
    RootRotationZ,
    RootRotationW,
    JointAngle(u32),
    JointVelocity(u32),
    SegmentGroundContact(u32),
}

#[derive(Clone, Copy, Debug, Default)]
pub struct JointSensorState {
    pub child_id: u32,
    pub angle_radians: f32,
    pub velocity_radians_per_second: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SegmentSensorState {
    pub segment_id: u32,
    pub ground_contact: f32,
}

#[derive(Clone, Debug, Default)]
pub struct BrainContext {
    pub time_seconds: f32,
    pub root_position: [f32; 3],
    pub root_linear_velocity: [f32; 3],
    pub root_angular_velocity: [f32; 3],
    pub root_rotation_xyzw: [f32; 4],
    pub joints: Vec<JointSensorState>,
    pub segments: Vec<SegmentSensorState>,
}

impl BrainContext {
    fn sensor(&self, sensor: SensorKind) -> f32 {
        match sensor {
            SensorKind::Time => self.time_seconds,
            SensorKind::RootHeight => self.root_position[1],
            SensorKind::RootVelocityX => self.root_linear_velocity[0],
            SensorKind::RootVelocityY => self.root_linear_velocity[1],
            SensorKind::RootVelocityZ => self.root_linear_velocity[2],
            SensorKind::RootAngularVelocityX => self.root_angular_velocity[0],
            SensorKind::RootAngularVelocityY => self.root_angular_velocity[1],
            SensorKind::RootAngularVelocityZ => self.root_angular_velocity[2],
            SensorKind::RootRotationX => self.root_rotation_xyzw[0],
            SensorKind::RootRotationY => self.root_rotation_xyzw[1],
            SensorKind::RootRotationZ => self.root_rotation_xyzw[2],
            SensorKind::RootRotationW => self.root_rotation_xyzw[3],
            SensorKind::JointAngle(child_id) => self
                .joints
                .iter()
                .find(|state| state.child_id == child_id)
                .map_or(0.0, |state| state.angle_radians),
            SensorKind::JointVelocity(child_id) => self
                .joints
                .iter()
                .find(|state| state.child_id == child_id)
                .map_or(0.0, |state| state.velocity_radians_per_second),
            SensorKind::SegmentGroundContact(segment_id) => self
                .segments
                .iter()
                .find(|state| state.segment_id == segment_id)
                .map_or(0.0, |state| state.ground_contact),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Expression {
    Constant(f32),
    Sensor(SensorKind),
    Add(Box<Expression>, Box<Expression>),
    Subtract(Box<Expression>, Box<Expression>),
    Multiply(Box<Expression>, Box<Expression>),
    Negate(Box<Expression>),
    Sin(Box<Expression>),
    Cos(Box<Expression>),
    Clamp {
        value: Box<Expression>,
        min: f32,
        max: f32,
    },
}

impl Expression {
    pub fn evaluate(&self, context: &BrainContext) -> f32 {
        sanitize(self.evaluate_inner(context))
    }

    fn evaluate_inner(&self, context: &BrainContext) -> f32 {
        match self {
            Self::Constant(value) => *value,
            Self::Sensor(sensor) => context.sensor(*sensor),
            Self::Add(left, right) => left.evaluate_inner(context) + right.evaluate_inner(context),
            Self::Subtract(left, right) => {
                left.evaluate_inner(context) - right.evaluate_inner(context)
            }
            Self::Multiply(left, right) => {
                left.evaluate_inner(context) * right.evaluate_inner(context)
            }
            Self::Negate(value) => -value.evaluate_inner(context),
            Self::Sin(value) => value.evaluate_inner(context).sin(),
            Self::Cos(value) => value.evaluate_inner(context).cos(),
            Self::Clamp { value, min, max } => value.evaluate_inner(context).clamp(*min, *max),
        }
    }

    pub fn node_count(&self) -> usize {
        match self {
            Self::Constant(_) | Self::Sensor(_) => 1,
            Self::Add(left, right) | Self::Subtract(left, right) | Self::Multiply(left, right) => {
                1 + left.node_count() + right.node_count()
            }
            Self::Negate(value) | Self::Sin(value) | Self::Cos(value) => 1 + value.node_count(),
            Self::Clamp { value, .. } => 1 + value.node_count(),
        }
    }

    pub fn sensor_node_count(&self) -> usize {
        match self {
            Self::Sensor(_) => 1,
            Self::Constant(_) => 0,
            Self::Add(left, right) | Self::Subtract(left, right) | Self::Multiply(left, right) => {
                left.sensor_node_count() + right.sensor_node_count()
            }
            Self::Negate(value) | Self::Sin(value) | Self::Cos(value) => value.sensor_node_count(),
            Self::Clamp { value, .. } => value.sensor_node_count(),
        }
    }

    pub fn collect_sensors(&self, sensors: &mut HashSet<SensorKind>) {
        match self {
            Self::Sensor(sensor) => {
                sensors.insert(*sensor);
            }
            Self::Constant(_) => {}
            Self::Add(left, right) | Self::Subtract(left, right) | Self::Multiply(left, right) => {
                left.collect_sensors(sensors);
                right.collect_sensors(sensors);
            }
            Self::Negate(value) | Self::Sin(value) | Self::Cos(value) => {
                value.collect_sensors(sensors);
            }
            Self::Clamp { value, .. } => value.collect_sensors(sensors),
        }
    }

    pub fn depth(&self) -> usize {
        match self {
            Self::Constant(_) | Self::Sensor(_) => 1,
            Self::Add(left, right) | Self::Subtract(left, right) | Self::Multiply(left, right) => {
                1 + left.depth().max(right.depth())
            }
            Self::Negate(value) | Self::Sin(value) | Self::Cos(value) => 1 + value.depth(),
            Self::Clamp { value, .. } => 1 + value.depth(),
        }
    }

    pub fn subtree_clone(&self, index: usize) -> Option<Expression> {
        let mut cursor = 0;
        self.subtree_clone_inner(index, &mut cursor)
    }

    fn subtree_clone_inner(&self, target: usize, cursor: &mut usize) -> Option<Expression> {
        if *cursor == target {
            return Some(self.clone());
        }
        *cursor += 1;

        match self {
            Self::Constant(_) | Self::Sensor(_) => None,
            Self::Add(left, right) | Self::Subtract(left, right) | Self::Multiply(left, right) => {
                left.subtree_clone_inner(target, cursor)
                    .or_else(|| right.subtree_clone_inner(target, cursor))
            }
            Self::Negate(value) | Self::Sin(value) | Self::Cos(value) => {
                value.subtree_clone_inner(target, cursor)
            }
            Self::Clamp { value, .. } => value.subtree_clone_inner(target, cursor),
        }
    }

    pub fn replace_subtree(&mut self, index: usize, replacement: Expression) -> bool {
        let mut cursor = 0;
        self.replace_subtree_inner(index, &replacement, &mut cursor)
    }

    fn replace_subtree_inner(
        &mut self,
        target: usize,
        replacement: &Expression,
        cursor: &mut usize,
    ) -> bool {
        if *cursor == target {
            *self = replacement.clone();
            return true;
        }
        *cursor += 1;

        match self {
            Self::Constant(_) | Self::Sensor(_) => false,
            Self::Add(left, right) | Self::Subtract(left, right) | Self::Multiply(left, right) => {
                left.replace_subtree_inner(target, replacement, cursor)
                    || right.replace_subtree_inner(target, replacement, cursor)
            }
            Self::Negate(value) | Self::Sin(value) | Self::Cos(value) => {
                value.replace_subtree_inner(target, replacement, cursor)
            }
            Self::Clamp { value, .. } => {
                value.replace_subtree_inner(target, replacement, cursor)
            }
        }
    }

    pub fn sanitize_sensor_targets(
        &mut self,
        joint_ids: &HashSet<u32>,
        segment_ids: &HashSet<u32>,
    ) {
        match self {
            Self::Sensor(SensorKind::JointAngle(child_id))
            | Self::Sensor(SensorKind::JointVelocity(child_id))
                if !joint_ids.contains(child_id) =>
            {
                *self = Self::Sensor(SensorKind::Time);
            }
            Self::Sensor(SensorKind::SegmentGroundContact(segment_id))
                if !segment_ids.contains(segment_id) =>
            {
                *self = Self::Sensor(SensorKind::Time);
            }
            Self::Add(left, right)
            | Self::Subtract(left, right)
            | Self::Multiply(left, right) => {
                left.sanitize_sensor_targets(joint_ids, segment_ids);
                right.sanitize_sensor_targets(joint_ids, segment_ids);
            }
            Self::Negate(value) | Self::Sin(value) | Self::Cos(value) => {
                value.sanitize_sensor_targets(joint_ids, segment_ids);
            }
            Self::Clamp { value, .. } => {
                value.sanitize_sensor_targets(joint_ids, segment_ids);
            }
            _ => {}
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.depth() > MAX_EXPRESSION_DEPTH {
            return Err(format!(
                "brain expression depth exceeds maximum of {MAX_EXPRESSION_DEPTH}"
            ));
        }

        match self {
            Self::Constant(value) if !value.is_finite() => {
                Err("brain constant must be finite".into())
            }
            Self::Clamp { value, min, max } => {
                if !min.is_finite() || !max.is_finite() || min >= max {
                    return Err("brain clamp bounds must be finite and min < max".into());
                }
                value.validate()
            }
            Self::Add(left, right) | Self::Subtract(left, right) | Self::Multiply(left, right) => {
                left.validate()?;
                right.validate()
            }
            Self::Negate(value) | Self::Sin(value) | Self::Cos(value) => value.validate(),
            Self::Constant(_) | Self::Sensor(_) => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BrainOutputGene {
    pub joint_child_id: u32,
    pub expression: Expression,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct BrainGenome {
    pub outputs: Vec<BrainOutputGene>,
}

impl BrainGenome {
    pub fn from_legacy_joints(joints: &[JointGene]) -> Self {
        Self {
            outputs: joints
                .iter()
                .map(|joint| BrainOutputGene {
                    joint_child_id: joint.child_id,
                    expression: legacy_expression(joint),
                })
                .collect(),
        }
    }

    pub fn output_for_joint(&self, child_id: u32) -> Option<&Expression> {
        self.outputs
            .iter()
            .find(|output| output.joint_child_id == child_id)
            .map(|output| &output.expression)
    }

    pub fn node_count(&self) -> usize {
        self.outputs
            .iter()
            .map(|output| output.expression.node_count())
            .sum()
    }

    pub fn sensor_node_count(&self) -> usize {
        self.outputs
            .iter()
            .map(|output| output.expression.sensor_node_count())
            .sum()
    }

    pub fn unique_sensor_count(&self) -> usize {
        let mut sensors = HashSet::new();
        for output in &self.outputs {
            output.expression.collect_sensors(&mut sensors);
        }
        sensors.len()
    }

    pub fn validate(&self, joints: &[JointGene], segments: &[SegmentGene]) -> Result<(), String> {
        let joint_ids: HashSet<u32> = joints.iter().map(|joint| joint.child_id).collect();
        let segment_ids: HashSet<u32> = segments.iter().map(|segment| segment.id).collect();
        let mut seen = HashSet::new();

        for output in &self.outputs {
            if !seen.insert(output.joint_child_id) {
                return Err(format!(
                    "brain has duplicate output for joint child {}",
                    output.joint_child_id
                ));
            }
            if !joint_ids.contains(&output.joint_child_id) {
                return Err(format!(
                    "brain output references missing joint child {}",
                    output.joint_child_id
                ));
            }
            output.expression.validate()?;

            let mut sensors = HashSet::new();
            output.expression.collect_sensors(&mut sensors);
            for sensor in sensors {
                match sensor {
                    SensorKind::JointAngle(child_id) | SensorKind::JointVelocity(child_id)
                        if !joint_ids.contains(&child_id) =>
                    {
                        return Err(format!(
                            "brain sensor references missing joint child {child_id}"
                        ));
                    }
                    SensorKind::SegmentGroundContact(segment_id)
                        if !segment_ids.contains(&segment_id) =>
                    {
                        return Err(format!(
                            "brain sensor references missing segment {segment_id}"
                        ));
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub fn sync_with_structure(&mut self, joints: &[JointGene], segments: &[SegmentGene]) {
        self.sync_with_joints(joints);
        let joint_ids: HashSet<u32> = joints.iter().map(|joint| joint.child_id).collect();
        let segment_ids: HashSet<u32> = segments.iter().map(|segment| segment.id).collect();
        for output in &mut self.outputs {
            output
                .expression
                .sanitize_sensor_targets(&joint_ids, &segment_ids);
        }
    }

    pub fn sync_with_joints(&mut self, joints: &[JointGene]) {
        self.outputs.retain(|output| {
            joints
                .iter()
                .any(|joint| joint.child_id == output.joint_child_id)
        });

        for joint in joints {
            if self.output_for_joint(joint.child_id).is_none() {
                self.outputs.push(BrainOutputGene {
                    joint_child_id: joint.child_id,
                    expression: legacy_expression(joint),
                });
            }
        }

        self.outputs.sort_by_key(|output| output.joint_child_id);
    }
}

pub fn legacy_expression(joint: &JointGene) -> Expression {
    use std::f32::consts::TAU;

    Expression::Clamp {
        value: Box::new(Expression::Multiply(
            Box::new(Expression::Constant(joint.motor_amplitude_radians)),
            Box::new(Expression::Sin(Box::new(Expression::Add(
                Box::new(Expression::Multiply(
                    Box::new(Expression::Constant(TAU * joint.motor_frequency_hz)),
                    Box::new(Expression::Sensor(SensorKind::Time)),
                )),
                Box::new(Expression::Constant(joint.motor_phase_radians)),
            )))),
        )),
        min: joint.limits_radians[0],
        max: joint.limits_radians[1],
    }
}

fn sanitize(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(-VALUE_LIMIT, VALUE_LIMIT)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{BrainContext, BrainGenome, Expression, JointSensorState, SensorKind};
    use crate::CreatureGenome;

    #[test]
    fn expression_tree_evaluates_targeted_joint_sensor() {
        let expression = Expression::Add(
            Box::new(Expression::Sensor(SensorKind::JointAngle(2))),
            Box::new(Expression::Constant(2.0)),
        );
        let value = expression.evaluate(&BrainContext {
            joints: vec![JointSensorState {
                child_id: 2,
                angle_radians: 0.5,
                velocity_radians_per_second: 0.0,
            }],
            ..BrainContext::default()
        });
        assert_eq!(value, 2.5);
    }

    #[test]
    fn subtree_can_be_cloned_and_replaced() {
        let mut expression = Expression::Add(
            Box::new(Expression::Constant(1.0)),
            Box::new(Expression::Sin(Box::new(Expression::Constant(2.0)))),
        );
        let donor = expression.subtree_clone(2).unwrap();
        assert_eq!(donor.node_count(), 2);
        assert!(expression.replace_subtree(1, Expression::Constant(9.0)));
        assert_eq!(expression.node_count(), 4);
    }

    #[test]
    fn legacy_brain_has_one_output_per_joint() {
        let creature = CreatureGenome::three_segment_walker();
        assert_eq!(creature.brain.outputs.len(), creature.joints.len());
        assert!(creature.brain.node_count() > creature.joints.len());
    }

    #[test]
    fn brain_sync_adds_missing_outputs() {
        let creature = CreatureGenome::three_segment_walker();
        let mut brain = BrainGenome::default();
        brain.sync_with_joints(&creature.joints);
        assert_eq!(brain.outputs.len(), creature.joints.len());
    }
}
