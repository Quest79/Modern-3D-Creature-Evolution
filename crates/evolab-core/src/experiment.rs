use serde::{Deserialize, Serialize};

use crate::{CreatureGenome, EvolutionConfig};

pub const EXPERIMENT_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ExperimentFile {
    pub format_version: u32,
    pub name: String,
    pub ancestor: CreatureGenome,
    pub evolution: EvolutionConfig,
}

impl ExperimentFile {
    pub fn new(
        name: impl Into<String>,
        ancestor: CreatureGenome,
        evolution: EvolutionConfig,
    ) -> Self {
        Self {
            format_version: EXPERIMENT_FORMAT_VERSION,
            name: name.into(),
            ancestor,
            evolution,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.format_version != EXPERIMENT_FORMAT_VERSION {
            return Err(format!(
                "unsupported experiment format version {} (expected {})",
                self.format_version, EXPERIMENT_FORMAT_VERSION
            ));
        }
        if self.name.trim().is_empty() {
            return Err("experiment name cannot be empty".into());
        }
        self.ancestor.validate()?;
        self.evolution.validate()
    }

    pub fn fork(&self, new_name: impl Into<String>) -> Self {
        let mut fork = self.clone();
        fork.name = new_name.into();
        fork
    }
}

#[cfg(test)]
mod tests {
    use super::ExperimentFile;
    use crate::{CreatureGenome, EvolutionConfig};

    #[test]
    fn experiment_round_trips_json() {
        let experiment = ExperimentFile::new(
            "Walker",
            CreatureGenome::three_segment_walker(),
            EvolutionConfig::default(),
        );
        experiment.validate().unwrap();

        let raw = serde_json::to_string(&experiment).unwrap();
        let restored: ExperimentFile = serde_json::from_str(&raw).unwrap();
        assert_eq!(experiment, restored);
    }

    #[test]
    fn fork_preserves_configuration() {
        let experiment = ExperimentFile::new(
            "Base",
            CreatureGenome::three_segment_walker(),
            EvolutionConfig::default(),
        );
        let fork = experiment.fork("Fork");
        assert_eq!(fork.name, "Fork");
        assert_eq!(fork.ancestor, experiment.ancestor);
        assert_eq!(fork.evolution, experiment.evolution);
    }
}
