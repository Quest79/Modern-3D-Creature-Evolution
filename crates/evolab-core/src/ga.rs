use rayon::{ThreadPoolBuilder, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{
    CreatureGenome, CreatureSimulator, GenomeRng, MutationConfig, SimulationConfig, mutate_genome,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EvolutionConfig {
    pub population_size: usize,
    pub generations: usize,
    pub tournament_size: usize,
    pub elite_count: usize,
    pub mutations_per_child: usize,
    pub seed: u64,
    pub worker_threads: usize,
    pub simulation: SimulationConfig,
    pub mutation: MutationConfig,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            population_size: 50,
            generations: 100,
            tournament_size: 7,
            elite_count: 2,
            mutations_per_child: 8,
            seed: 1,
            worker_threads: 0,
            simulation: SimulationConfig {
                duration_seconds: 5.0,
                ..SimulationConfig::default()
            },
            mutation: MutationConfig::default(),
        }
    }
}

impl EvolutionConfig {
    pub fn validate(&self) -> Result<(), String> {
        self.simulation.validate()?;
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
        if self.mutations_per_child == 0 {
            return Err("mutations_per_child must be greater than 0".into());
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EvaluatedCreature {
    pub genome: CreatureGenome,
    pub fitness: f32,
    pub distance_traveled: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GenerationSummary {
    pub generation: usize,
    pub best_fitness: f32,
    pub average_fitness: f32,
    pub worst_fitness: f32,
    pub best_distance: f32,
    pub best_segments: usize,
    pub best_joints: usize,
    pub evaluations_completed: usize,
    pub champion: CreatureGenome,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EvolutionResult {
    pub champion: CreatureGenome,
    pub champion_fitness: f32,
    pub champion_distance: f32,
    pub generations_completed: usize,
    pub evaluations_completed: usize,
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
    let mut population = initial_population(ancestor, config, &mut rng)?;
    let mut history = Vec::with_capacity(config.generations);
    let mut evaluations_completed = 0usize;
    let mut global_champion: Option<EvaluatedCreature> = None;

    for generation in 1..=config.generations {
        let mut evaluated = evaluate_population(&pool, &population, &config.simulation)?;
        evaluations_completed += evaluated.len();

        evaluated.sort_by(|a, b| b.fitness.total_cmp(&a.fitness));

        let best = evaluated
            .first()
            .ok_or_else(|| "evolution population unexpectedly empty".to_string())?;
        let worst = evaluated
            .last()
            .ok_or_else(|| "evolution population unexpectedly empty".to_string())?;
        let average =
            evaluated.iter().map(|item| item.fitness).sum::<f32>() / evaluated.len() as f32;

        if global_champion
            .as_ref()
            .is_none_or(|champion| best.fitness > champion.fitness)
        {
            global_champion = Some(best.clone());
        }

        let summary = GenerationSummary {
            generation,
            best_fitness: best.fitness,
            average_fitness: average,
            worst_fitness: worst.fitness,
            best_distance: best.distance_traveled,
            best_segments: best.genome.segments.len(),
            best_joints: best.genome.joints.len(),
            evaluations_completed,
            champion: best.genome.clone(),
        };

        on_generation(&summary)?;
        history.push(summary);

        if generation < config.generations {
            population = breed_next_generation(&evaluated, config, &mut rng)?;
        }
    }

    let champion = global_champion.ok_or_else(|| "evolution produced no champion".to_string())?;

    Ok(EvolutionResult {
        champion: champion.genome,
        champion_fitness: champion.fitness,
        champion_distance: champion.distance_traveled,
        generations_completed: config.generations,
        evaluations_completed,
        history,
    })
}

fn initial_population(
    ancestor: &CreatureGenome,
    config: &EvolutionConfig,
    rng: &mut GenomeRng,
) -> Result<Vec<CreatureGenome>, String> {
    let mut population = Vec::with_capacity(config.population_size);
    population.push(ancestor.clone());

    while population.len() < config.population_size {
        let seed = rng.next_seed();
        let child =
            mutate_genome(ancestor, seed, config.mutations_per_child, &config.mutation)?.genome;
        population.push(child);
    }

    Ok(population)
}

fn evaluate_population(
    pool: &rayon::ThreadPool,
    population: &[CreatureGenome],
    simulation: &SimulationConfig,
) -> Result<Vec<EvaluatedCreature>, String> {
    let results: Vec<Result<EvaluatedCreature, String>> = pool.install(|| {
        population
            .par_iter()
            .map(|genome| evaluate_creature(genome, simulation))
            .collect()
    });

    results.into_iter().collect()
}

fn evaluate_creature(
    genome: &CreatureGenome,
    simulation: &SimulationConfig,
) -> Result<EvaluatedCreature, String> {
    let simulator = CreatureSimulator;
    let mut ignore = |_snapshot: &crate::CreatureSnapshot| Ok(());
    let report = simulator.run_streaming(simulation, genome, usize::MAX, &mut ignore)?;

    let start = genome.segments[0].initial_position;
    let dx = report.final_root_position[0] - start[0];
    let dz = report.final_root_position[2] - start[2];
    let distance = (dx * dx + dz * dz).sqrt();

    Ok(EvaluatedCreature {
        genome: genome.clone(),
        fitness: distance,
        distance_traveled: distance,
    })
}

fn breed_next_generation(
    evaluated: &[EvaluatedCreature],
    config: &EvolutionConfig,
    rng: &mut GenomeRng,
) -> Result<Vec<CreatureGenome>, String> {
    let mut next = Vec::with_capacity(config.population_size);

    for elite in evaluated.iter().take(config.elite_count) {
        next.push(elite.genome.clone());
    }

    while next.len() < config.population_size {
        let parent_index = tournament_select(evaluated, config.tournament_size, rng);
        let parent = &evaluated[parent_index].genome;
        let child = mutate_genome(
            parent,
            rng.next_seed(),
            config.mutations_per_child,
            &config.mutation,
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
