use std::collections::{BTreeMap, HashSet};

use rayon::{ThreadPoolBuilder, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
    AcceleratorConfig, AcceleratorMode, CHECKPOINT_FORMAT_VERSION, ChampionArchiveEntry,
    CheckpointCandidate, ConditionContext, CreatureGenome, DiversitySummary,
    EffectiveEvolutionSettings, EvolutionCheckpoint, ExecutionPerformance, FitnessConfig,
    FitnessMetrics, FitnessResult, GenomeRng, LineageRecord, MapEliteCell, MutationConfig,
    MutationRecord, ParetoEntry, SimulationConfig, SpeciesSummary, TimelineConfig,
    TrialAggregation, crossover_brain_subtree, evaluate_fitness, mutate_genome,
    run_cuda_creature_batch,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EvolutionConfig {
    pub population_size: usize,
    /// Number of candidates evaluated each generation. 0 means population_size.
    /// This can be much larger than the survivor population on CUDA so idle GPU
    /// capacity is used to explore more offspring in the same generation.
    #[serde(default)]
    pub evaluation_pool_size: usize,
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
    pub accelerator: AcceleratorConfig,
    #[serde(default)]
    pub timeline: TimelineConfig,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            population_size: 50,
            evaluation_pool_size: 0,
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
            accelerator: AcceleratorConfig::default(),
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
        if self.evaluation_pool_size > 1_000_000 {
            return Err("evaluation_pool_size must be 0 or at most 1000000".into());
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
        self.accelerator.validate()?;
        self.timeline.validate()?;

        Ok(())
    }

    pub fn effective_evaluation_pool_size(&self) -> usize {
        if self.accelerator.mode == AcceleratorMode::Cpu || self.evaluation_pool_size == 0 {
            self.population_size
        } else {
            self.evaluation_pool_size.max(self.population_size)
        }
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
    pub individual_id: u64,
    pub parent_ids: Vec<u64>,
    pub species_id: u64,
    pub genome: CreatureGenome,
    pub fitness: f32,
    pub metrics: FitnessMetrics,
    pub trial_seeds: Vec<u64>,
    pub mutations: Vec<MutationRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GenerationSummary {
    pub generation: usize,
    pub best_fitness: f32,
    pub average_fitness: f32,
    pub median_fitness: f32,
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
    pub execution: ExecutionPerformance,
    pub effective_settings: EffectiveEvolutionSettings,
    pub active_timeline_events: Vec<String>,
    pub triggered_timeline_events: Vec<String>,
    pub diversity: DiversitySummary,
    pub species: Vec<SpeciesSummary>,
    pub lineage: Vec<LineageRecord>,
    pub pareto_front: Vec<ParetoEntry>,
    pub map_elites: Vec<MapEliteCell>,
    pub champion_id: u64,
    pub champion_parent_ids: Vec<u64>,
    pub champion_species_id: u64,
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
    pub champion_archive: Vec<ChampionArchiveEntry>,
    pub history: Vec<GenerationSummary>,
}

#[derive(Clone, Debug)]
struct Candidate {
    individual_id: u64,
    parent_ids: Vec<u64>,
    genome: CreatureGenome,
    mutations: Vec<MutationRecord>,
}

pub fn evolve_population<F>(
    ancestor: &CreatureGenome,
    config: &EvolutionConfig,
    on_generation: F,
) -> Result<EvolutionResult, String>
where
    F: FnMut(&GenerationSummary) -> Result<(), String>,
{
    evolve_population_checkpointed(ancestor, config, None, on_generation, |_checkpoint| Ok(()))
}

pub fn evolve_population_checkpointed<F, C>(
    ancestor: &CreatureGenome,
    config: &EvolutionConfig,
    resume: Option<&EvolutionCheckpoint>,
    mut on_generation: F,
    mut on_checkpoint: C,
) -> Result<EvolutionResult, String>
where
    F: FnMut(&GenerationSummary) -> Result<(), String>,
    C: FnMut(&EvolutionCheckpoint) -> Result<(), String>,
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

    let (
        start_generation,
        mut rng,
        mut active_condition_ids,
        mut next_individual_id,
        mut population,
        mut history,
        mut champion_archive,
        mut evaluations_completed,
        mut final_champion,
        mut final_settings,
    ) = if let Some(checkpoint) = resume {
        checkpoint.validate()?;
        if checkpoint.config != *config {
            return Err(
                "checkpoint configuration differs from the requested evolution configuration"
                    .into(),
            );
        }

        (
            checkpoint.next_generation,
            GenomeRng::from_state(checkpoint.rng_state),
            checkpoint
                .active_condition_ids
                .iter()
                .cloned()
                .collect::<HashSet<_>>(),
            checkpoint.next_individual_id,
            checkpoint
                .population
                .iter()
                .cloned()
                .map(Candidate::from)
                .collect::<Vec<_>>(),
            checkpoint.history.clone(),
            checkpoint.champion_archive.clone(),
            checkpoint.evaluations_completed,
            checkpoint.last_champion.clone(),
            checkpoint.final_settings.clone(),
        )
    } else {
        let mut rng = GenomeRng::new(config.seed);
        let active_condition_ids = HashSet::new();
        let mut first_settings = EffectiveEvolutionSettings::from(config);
        config
            .timeline
            .apply_to(1, &active_condition_ids, &mut first_settings);
        validate_effective_settings(&first_settings)?;

        let mut next_individual_id = 1_u64;
        let population =
            initial_population(ancestor, &first_settings, &mut rng, &mut next_individual_id)?;

        (
            1,
            rng,
            active_condition_ids,
            next_individual_id,
            population,
            Vec::with_capacity(config.generations),
            Vec::with_capacity(config.generations),
            0,
            None,
            first_settings,
        )
    };

    if start_generation > config.generations {
        return finish_evolution(
            config,
            final_champion,
            evaluations_completed,
            final_settings,
            champion_archive,
            history,
        );
    }

    for generation in start_generation..=config.generations {
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

        let evaluation_pool = expand_evaluation_pool(
            &population,
            config.effective_evaluation_pool_size(),
            config,
            &settings,
            &mut rng,
            &mut next_individual_id,
        )?;
        let (mut evaluated, execution) =
            evaluate_population(&pool, &evaluation_pool, &settings, &config.accelerator)?;
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
        let median_fitness = median(evaluated.iter().map(|item| item.fitness).collect());

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

        let diversity = summarize_diversity(&evaluated);
        let species = summarize_species(&evaluated);
        let lineage_count = settings.population_size.min(evaluated.len());
        let lineage = build_lineage_records(generation, &evaluated[..lineage_count]);
        let pareto_front = build_pareto_front(generation, &evaluated);
        let map_elites = build_map_elites(generation, &evaluated);

        champion_archive.push(ChampionArchiveEntry {
            generation,
            individual_id: best.individual_id,
            parent_ids: best.parent_ids.clone(),
            species_id: best.species_id,
            fitness: best.fitness,
            metrics: best.metrics,
            trial_seeds: best.trial_seeds.clone(),
            mutations: best.mutations.clone(),
            genome: best.genome.clone(),
        });

        let summary = GenerationSummary {
            generation,
            best_fitness: best.fitness,
            average_fitness: average,
            median_fitness,
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
            execution,
            effective_settings: settings.clone(),
            active_timeline_events,
            triggered_timeline_events: triggered,
            diversity,
            species,
            lineage,
            pareto_front,
            map_elites,
            champion_id: best.individual_id,
            champion_parent_ids: best.parent_ids.clone(),
            champion_species_id: best.species_id,
            champion: best.genome.clone(),
        };

        final_champion = Some(best.clone());
        final_settings = settings;
        history.push(summary);
        on_generation(
            history
                .last()
                .expect("generation summary was just appended"),
        )?;

        if generation < config.generations {
            let mut next_settings = EffectiveEvolutionSettings::from(config);
            config
                .timeline
                .apply_to(generation + 1, &active_condition_ids, &mut next_settings);
            validate_effective_settings(&next_settings)?;
            population = breed_next_generation(
                &evaluated,
                config,
                &next_settings,
                &mut rng,
                &mut next_individual_id,
            )?;
        } else {
            population.clear();
        }

        let mut active_ids: Vec<String> = active_condition_ids.iter().cloned().collect();
        active_ids.sort();
        let checkpoint = EvolutionCheckpoint {
            format_version: CHECKPOINT_FORMAT_VERSION,
            config: config.clone(),
            next_generation: generation + 1,
            rng_state: rng.state(),
            next_individual_id,
            active_condition_ids: active_ids,
            population: population
                .iter()
                .cloned()
                .map(CheckpointCandidate::from)
                .collect(),
            evaluations_completed,
            final_settings: final_settings.clone(),
            last_champion: final_champion.clone(),
            champion_archive: champion_archive.clone(),
            history: history.clone(),
        };
        checkpoint.validate()?;
        on_checkpoint(&checkpoint)?;
    }

    finish_evolution(
        config,
        final_champion,
        evaluations_completed,
        final_settings,
        champion_archive,
        history,
    )
}

fn finish_evolution(
    config: &EvolutionConfig,
    final_champion: Option<EvaluatedCreature>,
    evaluations_completed: usize,
    final_settings: EffectiveEvolutionSettings,
    champion_archive: Vec<ChampionArchiveEntry>,
    history: Vec<GenerationSummary>,
) -> Result<EvolutionResult, String> {
    let champion = final_champion.ok_or_else(|| "evolution produced no champion".to_string())?;

    Ok(EvolutionResult {
        champion: champion.genome,
        champion_fitness: champion.fitness,
        champion_distance: champion.metrics.distance,
        champion_metrics: champion.metrics,
        generations_completed: config.generations,
        evaluations_completed,
        final_settings,
        champion_archive,
        history,
    })
}

impl From<CheckpointCandidate> for Candidate {
    fn from(value: CheckpointCandidate) -> Self {
        Self {
            individual_id: value.individual_id,
            parent_ids: value.parent_ids,
            genome: value.genome,
            mutations: value.mutations,
        }
    }
}

impl From<Candidate> for CheckpointCandidate {
    fn from(value: Candidate) -> Self {
        Self {
            individual_id: value.individual_id,
            parent_ids: value.parent_ids,
            genome: value.genome,
            mutations: value.mutations,
        }
    }
}

fn initial_population(
    ancestor: &CreatureGenome,
    settings: &EffectiveEvolutionSettings,
    rng: &mut GenomeRng,
    next_individual_id: &mut u64,
) -> Result<Vec<Candidate>, String> {
    let mut population = Vec::with_capacity(settings.population_size);

    let ancestor_id = *next_individual_id;
    *next_individual_id += 1;
    population.push(Candidate {
        individual_id: ancestor_id,
        parent_ids: Vec::new(),
        genome: ancestor.clone(),
        mutations: Vec::new(),
    });

    while population.len() < settings.population_size {
        let mut accepted = None;
        for _ in 0..5 {
            let seed = rng.next_seed();
            if let Ok(mutation_result) = mutate_genome(
                ancestor,
                seed,
                settings.mutations_per_child,
                &settings.mutation,
            ) {
                accepted = Some(mutation_result);
                break;
            }
        }

        let Some(mutation_result) = accepted else {
            // Reject this offspring after five invalid attempts. The outer loop
            // immediately starts a fresh offspring attempt instead of stopping evolution.
            continue;
        };

        let individual_id = *next_individual_id;
        *next_individual_id += 1;
        population.push(Candidate {
            individual_id,
            parent_ids: vec![ancestor_id],
            genome: mutation_result.genome,
            mutations: mutation_result.mutations,
        });
    }

    Ok(population)
}

fn expand_evaluation_pool(
    population: &[Candidate],
    target_size: usize,
    config: &EvolutionConfig,
    settings: &EffectiveEvolutionSettings,
    rng: &mut GenomeRng,
    next_individual_id: &mut u64,
) -> Result<Vec<Candidate>, String> {
    if population.is_empty() {
        return Err("cannot expand an empty evolution population".into());
    }

    let target_size = target_size.max(population.len());
    let mut expanded = Vec::with_capacity(target_size);
    expanded.extend(population.iter().cloned());

    while expanded.len() < target_size {
        let parent = &population[rng.range_usize(population.len())];
        let mut parent_ids = vec![parent.individual_id];

        let base = if population.len() > 1 && rng.chance(config.crossover_chance) {
            let donor = &population[rng.range_usize(population.len())];
            if donor.individual_id != parent.individual_id {
                parent_ids.push(donor.individual_id);
            }
            crossover_brain_subtree(&parent.genome, &donor.genome, rng)
        } else {
            parent.genome.clone()
        };

        let mut accepted = None;
        for _ in 0..5 {
            if let Ok(mutation_result) = mutate_genome(
                &base,
                rng.next_seed(),
                settings.mutations_per_child,
                &settings.mutation,
            ) {
                accepted = Some(mutation_result);
                break;
            }
        }

        let Some(mutation_result) = accepted else {
            continue;
        };

        let individual_id = *next_individual_id;
        *next_individual_id += 1;
        expanded.push(Candidate {
            individual_id,
            parent_ids,
            genome: mutation_result.genome,
            mutations: mutation_result.mutations,
        });
    }

    Ok(expanded)
}

fn evaluate_population(
    pool: &rayon::ThreadPool,
    population: &[Candidate],
    settings: &EffectiveEvolutionSettings,
    accelerator: &AcceleratorConfig,
) -> Result<(Vec<EvaluatedCreature>, ExecutionPerformance), String> {
    let requested_mode = accelerator.mode;

    if requested_mode != AcceleratorMode::Cpu {
        let mut gpu_genomes = Vec::with_capacity(population.len() * settings.trials_per_creature);
        for candidate in population {
            for _ in 0..settings.trials_per_creature {
                gpu_genomes.push(candidate.genome.clone());
            }
        }

        match run_cuda_creature_batch(
            &gpu_genomes,
            &settings.simulation,
            &settings.fitness,
            accelerator,
        ) {
            Ok(batch) => {
                let mut evaluated = Vec::with_capacity(population.len());
                for (candidate_index, candidate) in population.iter().enumerate() {
                    let start = candidate_index * settings.trials_per_creature;
                    let end = start + settings.trials_per_creature;
                    let result =
                        aggregate_trials(&batch.fitness[start..end], settings.trial_aggregation);

                    let mut trial_seeds = Vec::with_capacity(settings.trials_per_creature);
                    for trial_index in 0..settings.trials_per_creature {
                        let seed = if trial_index == 0 {
                            settings.simulation.world.seed
                        } else {
                            trial_seed(settings.simulation.world.seed, trial_index)
                        };
                        trial_seeds.push(seed);
                    }

                    evaluated.push(EvaluatedCreature {
                        individual_id: candidate.individual_id,
                        parent_ids: candidate.parent_ids.clone(),
                        species_id: analysis_species_id(&candidate.genome),
                        genome: candidate.genome.clone(),
                        fitness: result.score,
                        metrics: result.metrics,
                        trial_seeds,
                        mutations: candidate.mutations.clone(),
                    });
                }
                return Ok((evaluated, batch.execution));
            }
            Err(error) if !accelerator.cpu_fallback => return Err(error),
            Err(_) => {}
        }
    }

    let started = std::time::Instant::now();
    let results: Vec<Result<EvaluatedCreature, String>> = pool.install(|| {
        population
            .par_iter()
            .map(|candidate| evaluate_creature(candidate, settings))
            .collect()
    });
    let evaluated: Vec<EvaluatedCreature> = results.into_iter().collect::<Result<_, _>>()?;
    let wall_seconds = started.elapsed().as_secs_f64().max(f64::EPSILON);
    let item_count = evaluated.len() * settings.trials_per_creature;
    let physics_steps = item_count as u64 * settings.simulation.step_count() as u64;
    let fallback_items = if requested_mode == AcceleratorMode::Cpu {
        0
    } else {
        item_count
    };

    let performance = ExecutionPerformance {
        requested_mode,
        actual_mode: AcceleratorMode::Cpu,
        cpu_items: item_count,
        gpu_items: 0,
        fallback_items,
        wall_seconds,
        items_per_second: item_count as f64 / wall_seconds,
        physics_steps_per_second: physics_steps as f64 / wall_seconds,
        devices: Vec::new(),
    };

    Ok((evaluated, performance))
}

fn evaluate_creature(
    candidate: &Candidate,
    settings: &EffectiveEvolutionSettings,
) -> Result<EvaluatedCreature, String> {
    let genome = &candidate.genome;
    let mut trials = Vec::with_capacity(settings.trials_per_creature);
    let mut trial_seeds = Vec::with_capacity(settings.trials_per_creature);

    for trial_index in 0..settings.trials_per_creature {
        let mut simulation = settings.simulation.clone();
        if trial_index > 0 {
            simulation.world.seed = trial_seed(simulation.world.seed, trial_index);
        }
        trial_seeds.push(simulation.world.seed);
        match evaluate_fitness(genome, &simulation, &settings.fitness) {
            Ok(result) => trials.push(result),
            Err(error) if error.starts_with("unstable physics:") => {
                trials.push(FitnessResult {
                    score: -1.0e30,
                    metrics: FitnessMetrics::default(),
                });
            }
            Err(error) => return Err(error),
        }
    }

    let result = aggregate_trials(&trials, settings.trial_aggregation);

    Ok(EvaluatedCreature {
        individual_id: candidate.individual_id,
        parent_ids: candidate.parent_ids.clone(),
        species_id: analysis_species_id(genome),
        genome: genome.clone(),
        fitness: result.score,
        metrics: result.metrics,
        trial_seeds,
        mutations: candidate.mutations.clone(),
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
    if values.len().is_multiple_of(2) {
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
    next_individual_id: &mut u64,
) -> Result<Vec<Candidate>, String> {
    let mut next = Vec::with_capacity(next_settings.population_size);
    let elite_count = config
        .elite_count
        .min(next_settings.population_size.saturating_sub(1))
        .max(1);
    let tournament_size = config.tournament_size.min(evaluated.len()).max(1);

    for elite in evaluated.iter().take(elite_count) {
        let individual_id = *next_individual_id;
        *next_individual_id += 1;
        next.push(Candidate {
            individual_id,
            parent_ids: vec![elite.individual_id],
            genome: elite.genome.clone(),
            mutations: Vec::new(),
        });
    }

    while next.len() < next_settings.population_size {
        let parent_index = tournament_select(evaluated, tournament_size, rng);
        let parent = &evaluated[parent_index];
        let mut parent_ids = vec![parent.individual_id];

        let base = if rng.chance(config.crossover_chance) {
            let donor_index = tournament_select(evaluated, tournament_size, rng);
            let donor = &evaluated[donor_index];
            if donor.individual_id != parent.individual_id {
                parent_ids.push(donor.individual_id);
            }
            crossover_brain_subtree(&parent.genome, &donor.genome, rng)
        } else {
            parent.genome.clone()
        };

        let mut accepted = None;
        for _ in 0..5 {
            if let Ok(mutation_result) = mutate_genome(
                &base,
                rng.next_seed(),
                next_settings.mutations_per_child,
                &next_settings.mutation,
            ) {
                accepted = Some(mutation_result);
                break;
            }
        }

        let Some(mutation_result) = accepted else {
            // Five invalid variants from this parent/base: discard this
            // offspring and start a fresh selection without aborting the run.
            continue;
        };

        let individual_id = *next_individual_id;
        *next_individual_id += 1;
        next.push(Candidate {
            individual_id,
            parent_ids,
            genome: mutation_result.genome,
            mutations: mutation_result.mutations,
        });
    }

    Ok(next)
}

fn analysis_species_id(genome: &CreatureGenome) -> u64 {
    let segment_count = genome.segments.len() as u64;
    let brain_bucket = (genome.brain.node_count() / 8) as u64;
    (segment_count << 32) | brain_bucket
}

fn morphology_signature(genome: &CreatureGenome) -> String {
    let mut parts = Vec::with_capacity(genome.segments.len());
    for segment in &genome.segments {
        parts.push(format!(
            "{:.2}:{:.2}:{:.2}",
            segment.half_extents[0], segment.half_extents[1], segment.half_extents[2]
        ));
    }
    format!(
        "{}|{}|{}",
        genome.segments.len(),
        genome.joints.len(),
        parts.join(",")
    )
}

fn summarize_diversity(evaluated: &[EvaluatedCreature]) -> DiversitySummary {
    if evaluated.is_empty() {
        return DiversitySummary::default();
    }

    let count = evaluated.len() as f32;
    let fitness_mean = evaluated.iter().map(|item| item.fitness).sum::<f32>() / count;
    let segment_mean = evaluated
        .iter()
        .map(|item| item.genome.segments.len() as f32)
        .sum::<f32>()
        / count;
    let brain_mean = evaluated
        .iter()
        .map(|item| item.genome.brain.node_count() as f32)
        .sum::<f32>()
        / count;

    let fitness_variance = evaluated
        .iter()
        .map(|item| {
            let delta = item.fitness - fitness_mean;
            delta * delta
        })
        .sum::<f32>()
        / count;
    let segment_variance = evaluated
        .iter()
        .map(|item| {
            let delta = item.genome.segments.len() as f32 - segment_mean;
            delta * delta
        })
        .sum::<f32>()
        / count;
    let brain_variance = evaluated
        .iter()
        .map(|item| {
            let delta = item.genome.brain.node_count() as f32 - brain_mean;
            delta * delta
        })
        .sum::<f32>()
        / count;

    let unique_morphologies = evaluated
        .iter()
        .map(|item| morphology_signature(&item.genome))
        .collect::<HashSet<_>>()
        .len();
    let analysis_species_count = evaluated
        .iter()
        .map(|item| item.species_id)
        .collect::<HashSet<_>>()
        .len();

    DiversitySummary {
        fitness_stddev: fitness_variance.sqrt(),
        segment_count_mean: segment_mean,
        segment_count_stddev: segment_variance.sqrt(),
        brain_nodes_mean: brain_mean,
        brain_nodes_stddev: brain_variance.sqrt(),
        unique_morphologies,
        analysis_species_count,
    }
}

fn summarize_species(evaluated: &[EvaluatedCreature]) -> Vec<SpeciesSummary> {
    let mut groups: BTreeMap<u64, Vec<&EvaluatedCreature>> = BTreeMap::new();
    for item in evaluated {
        groups.entry(item.species_id).or_default().push(item);
    }

    groups
        .into_iter()
        .map(|(species_id, members)| {
            let member_count = members.len();
            let best_fitness = members
                .iter()
                .map(|item| item.fitness)
                .max_by(f32::total_cmp)
                .unwrap_or(0.0);
            let average_fitness =
                members.iter().map(|item| item.fitness).sum::<f32>() / member_count as f32;
            let segment_count = members[0].genome.segments.len();
            let brain_node_bucket = members[0].genome.brain.node_count() / 8;

            SpeciesSummary {
                species_id,
                members: member_count,
                best_fitness,
                average_fitness,
                segment_count,
                brain_node_bucket,
            }
        })
        .collect()
}

fn build_lineage_records(generation: usize, evaluated: &[EvaluatedCreature]) -> Vec<LineageRecord> {
    evaluated
        .iter()
        .map(|item| LineageRecord {
            individual_id: item.individual_id,
            generation,
            parent_ids: item.parent_ids.clone(),
            species_id: item.species_id,
            fitness: item.fitness,
            metrics: item.metrics,
            segments: item.genome.segments.len(),
            joints: item.genome.joints.len(),
            brain_nodes: item.genome.brain.node_count(),
            trial_seeds: item.trial_seeds.clone(),
            mutations: item.mutations.clone(),
            genome: item.genome.clone(),
        })
        .collect()
}

fn build_pareto_front(generation: usize, evaluated: &[EvaluatedCreature]) -> Vec<ParetoEntry> {
    let mut front = Vec::new();

    for candidate in evaluated {
        let dominated = evaluated.iter().any(|other| {
            let no_worse = other.metrics.distance >= candidate.metrics.distance
                && other.metrics.energy <= candidate.metrics.energy;
            let strictly_better = other.metrics.distance > candidate.metrics.distance
                || other.metrics.energy < candidate.metrics.energy;
            no_worse && strictly_better
        });

        if !dominated {
            front.push(ParetoEntry {
                generation,
                individual_id: candidate.individual_id,
                species_id: candidate.species_id,
                fitness: candidate.fitness,
                metrics: candidate.metrics,
                segments: candidate.genome.segments.len(),
                brain_nodes: candidate.genome.brain.node_count(),
            });
        }
    }

    front.sort_by(|a, b| b.metrics.distance.total_cmp(&a.metrics.distance));
    front
}

fn build_map_elites(generation: usize, evaluated: &[EvaluatedCreature]) -> Vec<MapEliteCell> {
    let mut cells: BTreeMap<(usize, usize), &EvaluatedCreature> = BTreeMap::new();

    for item in evaluated {
        let key = (
            item.genome.segments.len(),
            item.genome.brain.node_count() / 8,
        );
        match cells.get(&key) {
            Some(existing) if existing.fitness >= item.fitness => {}
            _ => {
                cells.insert(key, item);
            }
        }
    }

    cells
        .into_iter()
        .map(|((segment_bin, brain_node_bin), item)| MapEliteCell {
            segment_bin,
            brain_node_bin,
            generation,
            individual_id: item.individual_id,
            species_id: item.species_id,
            fitness: item.fitness,
            metrics: item.metrics,
        })
        .collect()
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
    use super::{EvolutionConfig, GenerationSummary, evolve_population};
    use crate::{CreatureGenome, SimulationConfig};

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
        assert_eq!(result.champion_archive.len(), 3);
        assert_eq!(result.evaluations_completed, 18);
        assert_eq!(result.history[0].lineage.len(), 6);
        assert!(!result.history[0].lineage[0].trial_seeds.is_empty());
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

        let mut first_history = first.history.clone();
        let mut second_history = second.history.clone();
        clear_runtime_timing(&mut first_history);
        clear_runtime_timing(&mut second_history);
        assert_eq!(first_history, second_history);
    }
}
