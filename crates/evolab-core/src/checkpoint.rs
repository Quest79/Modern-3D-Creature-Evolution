use serde::{Deserialize, Serialize};

use crate::{
    ChampionArchiveEntry, CreatureGenome, EffectiveEvolutionSettings, EvaluatedCreature,
    EvolutionConfig, GenerationSummary, MutationRecord,
};

pub const CHECKPOINT_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CheckpointCandidate {
    pub individual_id: u64,
    pub parent_ids: Vec<u64>,
    pub genome: CreatureGenome,
    pub mutations: Vec<MutationRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EvolutionCheckpoint {
    pub format_version: u32,
    pub config: EvolutionConfig,
    pub next_generation: usize,
    pub rng_state: u64,
    pub next_individual_id: u64,
    pub active_condition_ids: Vec<String>,
    pub population: Vec<CheckpointCandidate>,
    pub evaluations_completed: usize,
    pub final_settings: EffectiveEvolutionSettings,
    pub last_champion: Option<EvaluatedCreature>,
    pub champion_archive: Vec<ChampionArchiveEntry>,
    pub history: Vec<GenerationSummary>,
}

impl EvolutionCheckpoint {
    pub fn validate(&self) -> Result<(), String> {
        if self.format_version != CHECKPOINT_FORMAT_VERSION {
            return Err(format!(
                "unsupported checkpoint format version {} (expected {})",
                self.format_version, CHECKPOINT_FORMAT_VERSION
            ));
        }
        self.config.validate()?;
        if self.next_generation == 0 || self.next_generation > self.config.generations + 1 {
            return Err("checkpoint next_generation is outside the experiment range".into());
        }
        if self.next_generation <= self.config.generations && self.population.is_empty() {
            return Err("checkpoint has no population to resume".into());
        }
        for candidate in &self.population {
            candidate.genome.validate()?;
        }
        if self.history.len() + 1 != self.next_generation {
            return Err(format!(
                "checkpoint history has {} generations but next_generation is {}",
                self.history.len(),
                self.next_generation
            ));
        }
        if self.champion_archive.len() != self.history.len() {
            return Err("checkpoint champion archive does not match history length".into());
        }
        Ok(())
    }

    pub fn completed_generations(&self) -> usize {
        self.next_generation.saturating_sub(1)
    }

    pub fn finished(&self) -> bool {
        self.next_generation > self.config.generations
    }
}

#[cfg(test)]
mod tests {
    use super::{CHECKPOINT_FORMAT_VERSION, EvolutionCheckpoint};
    use crate::{EffectiveEvolutionSettings, EvolutionConfig};

    #[test]
    fn empty_unstarted_checkpoint_requires_population() {
        let config = EvolutionConfig::default();
        let checkpoint = EvolutionCheckpoint {
            format_version: CHECKPOINT_FORMAT_VERSION,
            config: config.clone(),
            next_generation: 1,
            rng_state: 1,
            next_individual_id: 1,
            active_condition_ids: Vec::new(),
            population: Vec::new(),
            evaluations_completed: 0,
            final_settings: EffectiveEvolutionSettings::from(&config),
            last_champion: None,
            champion_archive: Vec::new(),
            history: Vec::new(),
        };
        assert!(checkpoint.validate().is_err());
    }
}
