use std::collections::HashSet;

use rayon::{ThreadPoolBuilder, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
    ConditionContext, CreatureGenome, EffectiveEvolutionSettings, FitnessConfig, FitnessMetrics,
    FitnessResult, GenomeRng, MutationConfig, SimulationConfig, TimelineConfig, TrialAggregation,
    crossover_brain_subtree, evaluate_fitness, mutate_genome,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EvolutionConfig {
    pub population_size: usize,
    pub generations: usize,
    pub tournament_size: usize,
    pub elite_count: usize,
    pub crossover_chance: f32,
    pub mutations_per_child: usize,
    pub seed: u64,
    pub worker_threads: usize,
    pub simulation: SimulationConfig,
    pub fitness: FitnessConfig,
    pub mutation: MutationConfig,
    pub trials_per_creature: usize,
    pub trial_aggregation: TrialAggregation,
    #[serde(default)]
    pub timeline: TimelineConfig,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            population_size: 50,
            generations: 100,
            tournament_size: 7,
            elite_count: 2,
            crossover_chance: 0.5,
            mutations_per_child: 8,
            seed: 1,
            worker_threads: 0,
            simulation: SimulationConfig {
                duration_seconds: 5.0,
                ..SimulationConfig::default()
            },
            fitness: FitnessConfig::default(),
            mutation: MutationConfig::default(),
            trials_per_creature: 1,
            trial_aggregation: TrialAggregation::Mean,
            timeline: TimelineConfig::default(),
        }
    }
}

impl EvolutionConfig {
    pub fn validate(&self) -> Result<(), String> {
        self.simulation.validate()?;
        self.fitness.validate()?;
        self.mutation.validate()?;

        if self.population_size < 2 {
            return Err("population_size must be at least 2".into());
        }
        if self.generations == 0 {
            return Err("generations must be greater than 0".into());
        }
        if self.tournament_size == 0 || self.tournament_size > self.population_size {
            return Err("tournament_size must be between 1 and population_size".into());
        }
        if self.elite_count == 0 || self.elite_count >= self.population_size {
            return Err("elite_count must be at least 1 and smaller than population_size".into());
        }
        if !self.crossover_chance.is_finite() || !(0.0..=1.0).contains(&self.crossover_chance) {
            return Err("crossover_chance must be between 0 and 1".into());
        }
        if self.mutations_per_child == 0 {
            return Err("mutations_per_child must be greater than 0".into());
        }
        if !(1..=100).contains(&self.trials_per_creature) {
            return Err("trials_per_creature must be between 1 and 100".into());
        }
        self.timeline.validate()?;

        Ok(())
    }
}

impl From<&EvolutionConfig> for EffectiveEvolutionSettings {
    fn from(config: &EvolutionConfig) -> Self {
        Self {
            population_size: config.population_size,
            mutations_per_child: config.mutations_per_child,
            mutation: config.mutation.clone(),
            simulation: config.simulation.clone(),
            fitness: config.fitness,
            trials_per_creature: config.trials_per_creature,
            trial_aggregation: config.trial_aggregation,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EvaluatedCreature {
    pub genome: CreatureGenome,
    pub fitness: f32,
    pub metrics: FitnessMetrics,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GenerationSummary {
    pub generation: usize,
    pub best_fitness: f32,
    pub average_fitness: f32,
    pub worst_fitness: f32,
    pub best_distance: f32,
    pub best_metrics: FitnessMetrics,
    pub best_segments: usize,
    pub best_joints: usize,
    pub best_brain_nodes: usize,
    pub best_brain_sensor_nodes: usize,
    pub best_brain_unique_sensors: usize,
    pub best_brain_outputs: usize,
    pub evaluations_completed: usize,
    pub effective_settings: EffectiveEvolutionSettings,
    pub active_timeline_events: Vec<String>,
    pub triggered_timeline_events: Vec<String>,
    pub champion: CreatureGenome,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EvolutionResult {
    pub champion: CreatureGenome,
    pub champion_fitness: f32,
    pub champion_distance: f32,
    pub champion_metrics: FitnessMetrics,
    pub generations_completed: usize,
    pub evaluations_completed: usize,
    pub final_settings: EffectiveEvolutionSettings,
    pub history: Vec<GenerationSummary>,
}

pub fn evolve_population<F>(
    ancestor: &CreatureGenome,
    config: &EvolutionConfig,
    mut on_generation: F,
) -> Result<EvolutionResult, String>
where
    F: FnMut(&GenerationSummary) -> Result<(), String>,
{
    config.validate()?;
    ancestor.validate()?;

    let pool = {
        let mut builder = ThreadPoolBuilder::new();
        if config.worker_threads > 0 {
            builder = builder.num_threads(config.worker_threads);
        }
        builder
            .build()
            .map_err(|err| format!("failed to create evolution worker pool: {err}"))?
    };

    let mut rng = GenomeRng::new(config.seed);
    let mut active_condition_ids = HashSet::new();
    let mut first_settings = EffectiveEvolutionSettings::from(config);
    config
        .timeline
        .apply_to(1, &active_condition_ids, &mut first_settings);
    validate_effective_settings(&first_settings)?;

    let mut population = initial_population(ancestor, &first_settings, &mut rng)?;
    let mut history = Vec::with_capacity(config.generations);
    let mut evaluations_completed = 0usize;
    let mut final_champion: Option<EvaluatedCreature> = None;
    let mut final_settings = first_settings.clone();

    for generation in 1..=config.generations {
        let mut settings = EffectiveEvolutionSettings::from(config);
        config
            .timeline
            .apply_to(generation, &active_condition_ids, &mut settings);
        validate_effective_settings(&settings)?;

        if population.len() != settings.population_size {
            return Err(format!(
                "timeline population mismatch at generation {generation}: expected {}, got {}",
                settings.population_size,
                population.len()
            ));
        }

        let mut evaluated = evaluate_population(&pool, &population, &settings)?;
        evaluations_completed += evaluated.len() * settings.trials_per_creature;

        evaluated.sort_by(|a, b| b.fitness.total_cmp(&a.fitness));

        let best = evaluated
            .first()
            .ok_or_else(|| "evolution population unexpectedly empty".to_string())?;
        let worst = evaluated
            .last()
            .ok_or_else(|| "evolution population unexpectedly empty".to_string())?;
        let average =
            evaluated.iter().map(|item| item.fitness).sum::<f32>() / evaluated.len() as f32;

        let context = ConditionContext {
            best_fitness: best.fitness,
            average_fitness: average,
            best_distance: best.metrics.distance,
        };
        let triggered = config
            .timeline
            .newly_triggered(generation, context, &active_condition_ids);
        for id in &triggered {
            active_condition_ids.insert(id.clone());
        }

        let mut active_timeline_events: Vec<String> =
            active_condition_ids.iter().cloned().collect();
        active_timeline_events.sort();

        let summary = GenerationSummary {
            generation,
            best_fitness: best.fitness,
            average_fitness: average,
            worst_fitness: worst.fitness,
            best_distance: best.metrics.distance,
            best_metrics: best.metrics,
            best_segments: best.genome.segments.len(),
            best_joints: best.genome.joints.len(),
            best_brain_nodes: best.genome.brain.node_count(),
            best_brain_sensor_nodes: best.genome.brain.sensor_node_count(),
            best_brain_unique_sensors: best.genome.brain.unique_sensor_count(),
            best_brain_outputs: best.genome.brain.outputs.len(),
            evaluations_completed,
            effective_settings: settings.clone(),
            active_timeline_events,
            triggered_timeline_events: triggered,
            champion: best.genome.clone(),
        };

        final_champion = Some(best.clone());
        final_settings = settings;
        on_generation(&summary)?;
        history.push(summary);

        if generation < config.generations {
            let mut next_settings = EffectiveEvolutionSettings::from(config);
            config
                .timeline
                .apply_to(generation + 1, &active_condition_ids, &mut next_settings);
            validate_effective_settings(&next_settings)?;
            population = breed_next_generation(&evaluated, config, &next_settings, &mut rng)?;
        }
    }

    let champion = final_champion.ok_or_else(|| "evolution produced no champion".to_string())?;

    Ok(EvolutionResult {
        champion: champion.genome,
        champion_fitness: champion.fitness,
        champion_distance: champion.metrics.distance,
        champion_metrics: champion.metrics,
        generations_completed: config.generations,
        evaluations_completed,
        final_settings,
        history,
    })
}

fn initial_population(
    ancestor: &CreatureGenome,
    settings: &EffectiveEvolutionSettings,
    rng: &mut GenomeRng,
) -> Result<Vec<CreatureGenome>, String> {
    let mut population = Vec::with_capacity(settings.population_size);
    population.push(ancestor.clone());

    while population.len() < settings.population_size {
        let seed = rng.next_seed();
        let child = mutate_genome(
            ancestor,
            seed,
            settings.mutations_per_child,
            &settings.mutation,
        )?
        .genome;
        population.push(child);
    }

    Ok(population)
}

fn evaluate_population(
    pool: &rayon::ThreadPool,
    population: &[CreatureGenome],
    settings: &EffectiveEvolutionSettings,
) -> Result<Vec<EvaluatedCreature>, String> {
    let results: Vec<Result<EvaluatedCreature, String>> = pool.install(|| {
        population
            .par_iter()
            .map(|genome| evaluate_creature(genome, settings))
            .collect()
    });

    results.into_iter().collect()
}

fn evaluate_creature(
    genome: &CreatureGenome,
    settings: &EffectiveEvolutionSettings,
) -> Result<EvaluatedCreature, String> {
    let mut trials = Vec::with_capacity(settings.trials_per_creature);

    for trial_index in 0..settings.trials_per_creature {
        let mut simulation = settings.simulation.clone();
        if trial_index > 0 {
            simulation.world.seed = trial_seed(simulation.world.seed, trial_index);
        }
        trials.push(evaluate_fitness(genome, &simulation, &settings.fitness)?);
    }

    let result = aggregate_trials(&trials, settings.trial_aggregation);

    Ok(EvaluatedCreature {
        genome: genome.clone(),
        fitness: result.score,
        metrics: result.metrics,
    })
}

fn trial_seed(base: u64, trial_index: usize) -> u64 {
    let mut value = base ^ (trial_index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
}

fn aggregate_trials(results: &[FitnessResult], aggregation: TrialAggregation) -> FitnessResult {
    debug_assert!(!results.is_empty());

    match aggregation {
        TrialAggregation::Best => *results
            .iter()
            .max_by(|a, b| a.score.total_cmp(&b.score))
            .expect("non-empty trial results"),
        TrialAggregation::Worst => *results
            .iter()
            .min_by(|a, b| a.score.total_cmp(&b.score))
            .expect("non-empty trial results"),
        TrialAggregation::Mean => {
            let count = results.len() as f32;
            let score = results.iter().map(|result| result.score).sum::<f32>() / count;
            let metrics = FitnessMetrics {
                distance: results
                    .iter()
                    .map(|result| result.metrics.distance)
                    .sum::<f32>()
                    / count,
                average_speed: results
                    .iter()
                    .map(|result| result.metrics.average_speed)
                    .sum::<f32>()
                    / count,
                upright: results
                    .iter()
                    .map(|result| result.metrics.upright)
                    .sum::<f32>()
                    / count,
                stability: results
                    .iter()
                    .map(|result| result.metrics.stability)
                    .sum::<f32>()
                    / count,
                energy: results
                    .iter()
                    .map(|result| result.metrics.energy)
                    .sum::<f32>()
                    / count,
            };
            FitnessResult { score, metrics }
        }
        TrialAggregation::Median => FitnessResult {
            score: median(results.iter().map(|result| result.score).collect()),
            metrics: FitnessMetrics {
                distance: median(
                    results
                        .iter()
                        .map(|result| result.metrics.distance)
                        .collect(),
                ),
                average_speed: median(
                    results
                        .iter()
                        .map(|result| result.metrics.average_speed)
                        .collect(),
                ),
                upright: median(
                    results
                        .iter()
                        .map(|result| result.metrics.upright)
                        .collect(),
                ),
                stability: median(
                    results
                        .iter()
                        .map(|result| result.metrics.stability)
                        .collect(),
                ),
                energy: median(results.iter().map(|result| result.metrics.energy).collect()),
            },
        },
    }
}

fn median(mut values: Vec<f32>) -> f32 {
    values.sort_by(f32::total_cmp);
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[mid - 1] + values[mid]) * 0.5
    } else {
        values[mid]
    }
}

fn validate_effective_settings(settings: &EffectiveEvolutionSettings) -> Result<(), String> {
    if settings.population_size < 2 {
        return Err("effective population_size must be at least 2".into());
    }
    if settings.mutations_per_child == 0 {
        return Err("effective mutations_per_child must be greater than 0".into());
    }
    if !(1..=100).contains(&settings.trials_per_creature) {
        return Err("effective trials_per_creature must be between 1 and 100".into());
    }
    settings.simulation.validate()?;
    settings.fitness.validate()?;
    settings.mutation.validate()?;
    Ok(())
}

fn breed_next_generation(
    evaluated: &[EvaluatedCreature],
    config: &EvolutionConfig,
    next_settings: &EffectiveEvolutionSettings,
    rng: &mut GenomeRng,
) -> Result<Vec<CreatureGenome>, String> {
    let mut next = Vec::with_capacity(next_settings.population_size);
    let elite_count = config
        .elite_count
        .min(next_settings.population_size.saturating_sub(1))
        .max(1);
    let tournament_size = config.tournament_size.min(evaluated.len()).max(1);

    for elite in evaluated.iter().take(elite_count) {
        next.push(elite.genome.clone());
    }

    while next.len() < next_settings.population_size {
        let parent_index = tournament_select(evaluated, tournament_size, rng);
        let parent = &evaluated[parent_index].genome;

        let base = if rng.chance(config.crossover_chance) {
            let donor_index = tournament_select(evaluated, tournament_size, rng);
            let donor = &evaluated[donor_index].genome;
            crossover_brain_subtree(parent, donor, rng)
        } else {
            parent.clone()
        };

        let child = mutate_genome(
            &base,
            rng.next_seed(),
            next_settings.mutations_per_child,
            &next_settings.mutation,
        )?
        .genome;
        next.push(child);
    }

    Ok(next)
}

fn tournament_select(
    evaluated: &[EvaluatedCreature],
    tournament_size: usize,
    rng: &mut GenomeRng,
) -> usize {
    let mut best_index = rng.range_usize(evaluated.len());

    for _ in 1..tournament_size {
        let candidate = rng.range_usize(evaluated.len());
        if evaluated[candidate].fitness > evaluated[best_index].fitness {
            best_index = candidate;
        }
    }

    best_index
}

#[cfg(test)]
mod tests {
    use super::{EvolutionConfig, evolve_population};
    use crate::{CreatureGenome, SimulationConfig};

    #[test]
    fn short_evolution_returns_generation_history() {
        let config = EvolutionConfig {
            population_size: 6,
            generations: 3,
            tournament_size: 3,
            elite_count: 1,
            mutations_per_child: 2,
            worker_threads: 2,
            simulation: SimulationConfig {
                duration_seconds: 0.10,
                ..SimulationConfig::default()
            },
            ..EvolutionConfig::default()
        };

        let result =
            evolve_population(&CreatureGenome::three_segment_walker(), &config, |_| Ok(()))
                .unwrap();

        assert_eq!(result.history.len(), 3);
        assert_eq!(result.evaluations_completed, 18);
        assert!(result.champion.validate().is_ok());
    }

    #[test]
    fn evolution_is_reproducible_for_fixed_seed() {
        let config = EvolutionConfig {
            population_size: 5,
            generations: 2,
            tournament_size: 2,
            elite_count: 1,
            mutations_per_child: 2,
            worker_threads: 2,
            seed: 77,
            simulation: SimulationConfig {
                duration_seconds: 0.08,
                ..SimulationConfig::default()
            },
            ..EvolutionConfig::default()
        };

        let first = evolve_population(&CreatureGenome::three_segment_walker(), &config, |_| Ok(()))
            .unwrap();
        let second =
            evolve_population(&CreatureGenome::three_segment_walker(), &config, |_| Ok(()))
                .unwrap();

        assert_eq!(first.champion, second.champion);
        assert_eq!(first.champion_fitness, second.champion_fitness);
        assert_eq!(first.history, second.history);
    }
}
