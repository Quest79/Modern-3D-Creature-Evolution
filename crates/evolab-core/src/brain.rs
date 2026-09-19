use serde::{Deserialize, Serialize};

use crate::JointGene;

const MAX_EXPRESSION_DEPTH: usize = 16;
const VALUE_LIMIT: f32 = 1000.0;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, Default)]
pub struct BrainContext {
    pub time_seconds: f32,
    pub root_position: [f32; 3],
    pub root_linear_velocity: [f32; 3],
    pub root_angular_velocity: [f32; 3],
    pub root_rotation_xyzw: [f32; 4],
}

impl BrainContext {
    fn sensor(self, sensor: SensorKind) -> f32 {
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
        }
    }
}

impl Expression {
    pub fn evaluate(&self, context: BrainContext) -> f32 {
        sanitize(self.evaluate_inner(context))
    }

    fn evaluate_inner(&self, context: BrainContext) -> f32 {
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
            Self::Add(left, right)
            | Self::Subtract(left, right)
            | Self::Multiply(left, right) => {
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

    pub fn validate(&self, joints: &[JointGene]) -> Result<(), String> {
        let mut seen = std::collections::HashSet::new();

        for output in &self.outputs {
            if !seen.insert(output.joint_child_id) {
                return Err(format!(
                    "brain has duplicate output for joint child {}",
                    output.joint_child_id
                ));
            }
            if !joints
                .iter()
                .any(|joint| joint.child_id == output.joint_child_id)
            {
                return Err(format!(
                    "brain output references missing joint child {}",
                    output.joint_child_id
                ));
            }
            output.expression.validate()?;
        }

        Ok(())
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
    use super::{BrainContext, BrainGenome, Expression, SensorKind};
    use crate::CreatureGenome;

    #[test]
    fn expression_tree_evaluates_sensors_and_math() {
        let expression = Expression::Add(
            Box::new(Expression::Sensor(SensorKind::RootVelocityX)),
            Box::new(Expression::Constant(2.0)),
        );
        let value = expression.evaluate(BrainContext {
            root_linear_velocity: [3.0, 0.0, 0.0],
            ..BrainContext::default()
        });
        assert_eq!(value, 5.0);
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
