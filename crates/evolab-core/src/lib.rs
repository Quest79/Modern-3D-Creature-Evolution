//! Core simulation and execution primitives for Modern 3D Creature Evolution.
//!
//! Rendering is intentionally outside this crate. The GUI consumes backend-
//! neutral snapshots/results and never owns the authoritative physics state.

mod backend;
mod batch;
mod brain;
mod config;
mod creature;
mod evolution;
mod experiment;
mod fitness;
mod ga;
mod rapier_cpu;
mod results;
mod timeline;
mod world;

pub use backend::{
    BackendCapabilities, PhysicsBackend, ProbeSpec, SimulationReport, WorldSnapshot,
};
pub use batch::BatchRunner;
pub use brain::{
    BrainContext, BrainGenome, BrainOutputGene, Expression, JointSensorState, SegmentSensorState,
    SensorKind, legacy_expression,
};
pub use config::SimulationConfig;
pub use creature::{
    CreatureBodySnapshot, CreatureGenome, CreatureReport, CreatureSimulator, CreatureSnapshot,
    JointGene, SegmentGene,
};
pub use evolution::{
    GenomeRng, MutationConfig, MutationKind, MutationRecord, MutationResult,
    crossover_brain_subtree, mutate_genome, random_creature,
};
pub use experiment::{EXPERIMENT_FORMAT_VERSION, ExperimentFile};
pub use fitness::{FitnessConfig, FitnessMetrics, FitnessResult, FitnessWeights, evaluate_fitness};
pub use ga::{
    EvaluatedCreature, EvolutionConfig, EvolutionResult, GenerationSummary, evolve_population,
};
pub use rapier_cpu::RapierCpuBackend;
pub use results::{
    ChampionArchiveEntry, DiversitySummary, EvolutionResultsFile, LineageRecord, MapEliteCell,
    ParetoEntry, RESULTS_FORMAT_VERSION, SpeciesSummary,
};
pub use timeline::{
    ConditionContext, EffectiveEvolutionSettings, TimelineChanges, TimelineCondition,
    TimelineConfig, TimelineKeyframe, TrialAggregation,
};
pub use world::{TerrainKind, WorldBox, WorldBoxKind, WorldConfig};
