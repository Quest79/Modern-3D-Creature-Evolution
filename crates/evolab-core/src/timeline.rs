use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::{FitnessConfig, MutationConfig, SimulationConfig, WorldConfig};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrialAggregation {
    #[default]
    Mean,
    Median,
    Worst,
    Best,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ConditionContext {
    pub best_fitness: f32,
    pub average_fitness: f32,
    pub best_distance: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum TimelineCondition {
    BestFitnessAtLeast(f32),
    AverageFitnessAtLeast(f32),
    BestDistanceAtLeast(f32),
}

impl TimelineCondition {
    pub fn validate(&self) -> Result<(), String> {
        let value = match self {
            Self::BestFitnessAtLeast(value)
            | Self::AverageFitnessAtLeast(value)
            | Self::BestDistanceAtLeast(value) => *value,
        };
        if !value.is_finite() {
            return Err("timeline condition threshold must be finite".into());
        }
        Ok(())
    }

    pub fn is_satisfied(&self, context: ConditionContext) -> bool {
        match self {
            Self::BestFitnessAtLeast(value) => context.best_fitness >= *value,
            Self::AverageFitnessAtLeast(value) => context.average_fitness >= *value,
            Self::BestDistanceAtLeast(value) => context.best_distance >= *value,
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct TimelineChanges {
    #[serde(default)]
    pub world: Option<WorldConfig>,
    #[serde(default)]
    pub fitness: Option<FitnessConfig>,
    #[serde(default)]
    pub population_size: Option<usize>,
    #[serde(default)]
    pub mutations_per_child: Option<usize>,
    #[serde(default)]
    pub structural_mutation_chance: Option<f32>,
    #[serde(default)]
    pub motor_strength_multiplier: Option<f32>,
    #[serde(default)]
    pub trial_duration_seconds: Option<f32>,
    #[serde(default)]
    pub trials_per_creature: Option<usize>,
    #[serde(default)]
    pub trial_aggregation: Option<TrialAggregation>,
}

impl TimelineChanges {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(world) = &self.world {
            world.validate()?;
        }
        if let Some(fitness) = &self.fitness {
            fitness.validate()?;
        }
        if let Some(value) = self.population_size
            && value < 2
        {
            return Err("timeline population_size must be at least 2".into());
        }
        if let Some(value) = self.mutations_per_child
            && value == 0
        {
            return Err("timeline mutations_per_child must be greater than 0".into());
        }
        if let Some(value) = self.structural_mutation_chance
            && (!value.is_finite() || !(0.0..=1.0).contains(&value))
        {
            return Err("timeline structural_mutation_chance must be between 0 and 1".into());
        }
        if let Some(value) = self.motor_strength_multiplier
            && (!value.is_finite() || !(0.0..=1.0).contains(&value))
        {
            return Err("timeline motor_strength_multiplier is biological activation and must be between 0 and 1".into());
        }
        if let Some(value) = self.trial_duration_seconds
            && (!value.is_finite() || value <= 0.0)
        {
            return Err("timeline trial_duration_seconds must be greater than 0".into());
        }
        if let Some(value) = self.trials_per_creature
            && !(1..=100).contains(&value)
        {
            return Err("timeline trials_per_creature must be between 1 and 100".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimelineKeyframe {
    pub id: String,
    pub generation: usize,
    #[serde(default)]
    pub condition: Option<TimelineCondition>,
    #[serde(default)]
    pub changes: TimelineChanges,
}

impl TimelineKeyframe {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("timeline keyframe id cannot be empty".into());
        }
        if self.generation == 0 {
            return Err("timeline generation must be at least 1".into());
        }
        if let Some(condition) = &self.condition {
            condition.validate()?;
        }
        self.changes.validate()
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct TimelineConfig {
    #[serde(default)]
    pub keyframes: Vec<TimelineKeyframe>,
}

impl TimelineConfig {
    pub fn validate(&self) -> Result<(), String> {
        let mut ids = HashSet::new();
        for keyframe in &self.keyframes {
            keyframe.validate()?;
            if !ids.insert(keyframe.id.as_str()) {
                return Err(format!("duplicate timeline keyframe id: {}", keyframe.id));
            }
        }
        Ok(())
    }

    pub fn newly_triggered(
        &self,
        generation: usize,
        context: ConditionContext,
        active_ids: &HashSet<String>,
    ) -> Vec<String> {
        self.keyframes
            .iter()
            .filter(|keyframe| {
                keyframe.generation <= generation
                    && keyframe.condition.is_some()
                    && !active_ids.contains(&keyframe.id)
                    && keyframe
                        .condition
                        .as_ref()
                        .is_some_and(|condition| condition.is_satisfied(context))
            })
            .map(|keyframe| keyframe.id.clone())
            .collect()
    }

    pub fn apply_to(
        &self,
        generation: usize,
        active_condition_ids: &HashSet<String>,
        settings: &mut EffectiveEvolutionSettings,
    ) {
        let mut indexed: Vec<(usize, &TimelineKeyframe)> =
            self.keyframes.iter().enumerate().collect();
        indexed.sort_by_key(|(index, keyframe)| (keyframe.generation, *index));

        for (_, keyframe) in indexed {
            let active = if keyframe.condition.is_some() {
                active_condition_ids.contains(&keyframe.id)
            } else {
                keyframe.generation <= generation
            };

            if active {
                settings.apply_changes(&keyframe.changes);
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EffectiveEvolutionSettings {
    pub population_size: usize,
    pub mutations_per_child: usize,
    pub mutation: MutationConfig,
    pub simulation: SimulationConfig,
    pub fitness: FitnessConfig,
    pub trials_per_creature: usize,
    pub trial_aggregation: TrialAggregation,
}

impl EffectiveEvolutionSettings {
    pub fn apply_changes(&mut self, changes: &TimelineChanges) {
        if let Some(world) = &changes.world {
            self.simulation.world = world.clone();
        }
        if let Some(fitness) = changes.fitness {
            self.fitness = fitness;
        }
        if let Some(value) = changes.population_size {
            self.population_size = value;
        }
        if let Some(value) = changes.mutations_per_child {
            self.mutations_per_child = value;
        }
        if let Some(value) = changes.structural_mutation_chance {
            self.mutation.structural_mutation_chance = value;
        }
        if let Some(value) = changes.motor_strength_multiplier {
            self.simulation.motor_strength_multiplier = value;
        }
        if let Some(value) = changes.trial_duration_seconds {
            self.simulation.duration_seconds = value;
        }
        if let Some(value) = changes.trials_per_creature {
            self.trials_per_creature = value;
        }
        if let Some(value) = changes.trial_aggregation {
            self.trial_aggregation = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        ConditionContext, EffectiveEvolutionSettings, TimelineChanges, TimelineCondition,
        TimelineConfig, TimelineKeyframe, TrialAggregation,
    };
    use crate::{EvolutionConfig, WorldConfig};

    #[test]
    fn scheduled_keyframe_applies_at_generation() {
        let timeline = TimelineConfig {
            keyframes: vec![TimelineKeyframe {
                id: "g5".into(),
                generation: 5,
                condition: None,
                changes: TimelineChanges {
                    population_size: Some(80),
                    trial_duration_seconds: Some(9.0),
                    ..TimelineChanges::default()
                },
            }],
        };
        let base = EvolutionConfig::default();
        let mut settings = EffectiveEvolutionSettings::from(&base);
        timeline.apply_to(4, &HashSet::new(), &mut settings);
        assert_eq!(settings.population_size, 50);

        timeline.apply_to(5, &HashSet::new(), &mut settings);
        assert_eq!(settings.population_size, 80);
        assert_eq!(settings.simulation.duration_seconds, 9.0);
    }

    #[test]
    fn conditional_keyframe_waits_for_activation() {
        let timeline = TimelineConfig {
            keyframes: vec![TimelineKeyframe {
                id: "fast".into(),
                generation: 2,
                condition: Some(TimelineCondition::BestFitnessAtLeast(3.0)),
                changes: TimelineChanges {
                    trials_per_creature: Some(4),
                    trial_aggregation: Some(TrialAggregation::Worst),
                    world: Some(WorldConfig {
                        ground_friction: 0.5,
                        ..WorldConfig::default()
                    }),
                    ..TimelineChanges::default()
                },
            }],
        };
        let active = HashSet::new();
        let triggered = timeline.newly_triggered(
            2,
            ConditionContext {
                best_fitness: 3.5,
                ..ConditionContext::default()
            },
            &active,
        );
        assert_eq!(triggered, vec!["fast".to_string()]);
    }
}
