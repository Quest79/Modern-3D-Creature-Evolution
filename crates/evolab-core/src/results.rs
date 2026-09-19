use serde::{Deserialize, Serialize};

use crate::{CreatureGenome, EvolutionConfig, EvolutionResult, FitnessMetrics, MutationRecord};

pub const RESULTS_FORMAT_VERSION: u32 = 2;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct DiversitySummary {
    pub fitness_stddev: f32,
    pub segment_count_mean: f32,
    pub segment_count_stddev: f32,
    pub brain_nodes_mean: f32,
    pub brain_nodes_stddev: f32,
    pub unique_morphologies: usize,
    pub analysis_species_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SpeciesSummary {
    pub species_id: u64,
    pub members: usize,
    pub best_fitness: f32,
    pub average_fitness: f32,
    pub segment_count: usize,
    pub brain_node_bucket: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LineageRecord {
    pub individual_id: u64,
    pub generation: usize,
    pub parent_ids: Vec<u64>,
    pub species_id: u64,
    pub fitness: f32,
    pub metrics: FitnessMetrics,
    pub segments: usize,
    pub joints: usize,
    pub brain_nodes: usize,
    pub trial_seeds: Vec<u64>,
    pub mutations: Vec<MutationRecord>,
    pub genome: CreatureGenome,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChampionArchiveEntry {
    pub generation: usize,
    pub individual_id: u64,
    pub parent_ids: Vec<u64>,
    pub species_id: u64,
    pub fitness: f32,
    pub metrics: FitnessMetrics,
    pub trial_seeds: Vec<u64>,
    pub mutations: Vec<MutationRecord>,
    pub genome: CreatureGenome,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ParetoEntry {
    pub generation: usize,
    pub individual_id: u64,
    pub species_id: u64,
    pub fitness: f32,
    pub metrics: FitnessMetrics,
    pub segments: usize,
    pub brain_nodes: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MapEliteCell {
    pub segment_bin: usize,
    pub brain_node_bin: usize,
    pub generation: usize,
    pub individual_id: u64,
    pub species_id: u64,
    pub fitness: f32,
    pub metrics: FitnessMetrics,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EvolutionResultsFile {
    pub format_version: u32,
    pub experiment_name: String,
    pub evolution: EvolutionConfig,
    pub wall_seconds: f64,
    pub result: EvolutionResult,
}

impl EvolutionResultsFile {
    pub fn new(
        experiment_name: impl Into<String>,
        evolution: EvolutionConfig,
        wall_seconds: f64,
        result: EvolutionResult,
    ) -> Self {
        Self {
            format_version: RESULTS_FORMAT_VERSION,
            experiment_name: experiment_name.into(),
            evolution,
            wall_seconds,
            result,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.format_version != RESULTS_FORMAT_VERSION {
            return Err(format!(
                "unsupported results format version {} (expected {})",
                self.format_version, RESULTS_FORMAT_VERSION
            ));
        }
        if self.experiment_name.trim().is_empty() {
            return Err("results experiment_name cannot be empty".into());
        }
        if !self.wall_seconds.is_finite() || self.wall_seconds < 0.0 {
            return Err("results wall_seconds must be finite and non-negative".into());
        }
        self.evolution.validate()?;
        self.result.champion.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::EvolutionResultsFile;
    use crate::{CreatureGenome, EvolutionConfig, GenerationSummary, evolve_population};

    fn assert_close(left: f64, right: f64) {
        let tolerance = 1e-12 * left.abs().max(right.abs()).max(1.0);
        assert!(
            (left - right).abs() <= tolerance,
            "floating telemetry changed too much: {left} != {right}"
        );
    }

    fn clear_runtime_timing(history: &mut [GenerationSummary]) {
        for summary in history {
            summary.execution.wall_seconds = 0.0;
            summary.execution.items_per_second = 0.0;
            summary.execution.physics_steps_per_second = 0.0;
            for device in &mut summary.execution.devices {
                device.wall_seconds = 0.0;
                device.items_per_second = 0.0;
                device.physics_steps_per_second = 0.0;
            }
        }
    }

    #[test]
    fn results_file_round_trips_json() {
        let config = EvolutionConfig {
            population_size: 4,
            generations: 1,
            tournament_size: 2,
            elite_count: 1,
            simulation: crate::SimulationConfig {
                duration_seconds: 0.05,
                ..crate::SimulationConfig::default()
            },
            ..EvolutionConfig::default()
        };
        let result =
            evolve_population(&CreatureGenome::three_segment_walker(), &config, |_| Ok(()))
                .unwrap();
        let file = EvolutionResultsFile::new("Test", config, 0.25, result);
        file.validate().unwrap();

        let raw = serde_json::to_string(&file).unwrap();
        let restored: EvolutionResultsFile = serde_json::from_str(&raw).unwrap();

        assert_eq!(file.result.history.len(), restored.result.history.len());
        for (expected, actual) in file.result.history.iter().zip(&restored.result.history) {
            assert_close(
                expected.execution.wall_seconds,
                actual.execution.wall_seconds,
            );
            assert_close(
                expected.execution.items_per_second,
                actual.execution.items_per_second,
            );
            assert_close(
                expected.execution.physics_steps_per_second,
                actual.execution.physics_steps_per_second,
            );
            assert_eq!(
                expected.execution.devices.len(),
                actual.execution.devices.len()
            );
            for (expected_device, actual_device) in expected
                .execution
                .devices
                .iter()
                .zip(&actual.execution.devices)
            {
                assert_eq!(expected_device.device_id, actual_device.device_id);
                assert_eq!(expected_device.items, actual_device.items);
                assert_close(expected_device.wall_seconds, actual_device.wall_seconds);
                assert_close(
                    expected_device.items_per_second,
                    actual_device.items_per_second,
                );
                assert_close(
                    expected_device.physics_steps_per_second,
                    actual_device.physics_steps_per_second,
                );
            }
        }

        let mut expected = file;
        let mut actual = restored;
        clear_runtime_timing(&mut expected.result.history);
        clear_runtime_timing(&mut actual.result.history);
        assert_eq!(expected, actual);
    }
}
