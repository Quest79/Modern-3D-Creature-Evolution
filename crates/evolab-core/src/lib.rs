//! Core simulation and execution primitives for Modern 3D Creature Evolution.
//!
//! Rendering is intentionally outside this crate. The GUI consumes backend-
//! neutral snapshots/results and never owns the authoritative physics state.

mod accelerator;
mod backend;
mod batch;
mod brain;
mod checkpoint;
mod config;
mod creature;
mod cuda_probe;
mod evolution;
mod experiment;
mod fitness;
mod ga;
mod rapier_cpu;
mod results;
mod timeline;
mod world;

pub use accelerator::{
    AcceleratorConfig, AcceleratorMode, CudaDeviceInfo, DevicePerformance, DeviceWorkAssignment,
    ExecutionPerformance, GpuCompatibility, ThroughputMode, schedule_gpu_work,
};
pub use backend::{
    BackendCapabilities, PhysicsBackend, ProbeSpec, SimulationReport, WorldSnapshot,
};
pub use batch::BatchRunner;
pub use brain::{
    BrainContext, BrainGenome, BrainOutputGene, Expression, JointSensorState, SegmentSensorState,
    SensorKind, legacy_expression,
};
pub use checkpoint::{CHECKPOINT_FORMAT_VERSION, CheckpointCandidate, EvolutionCheckpoint};
pub use config::SimulationConfig;
pub use creature::{
    BIOLOGICAL_MAX_CONTACT_FRICTION, BIOLOGICAL_MAX_CYCLIC_POWER_W_PER_KG,
    BIOLOGICAL_MAX_DENSITY_KG_M3, BIOLOGICAL_MAX_MUSCLE_STRESS_PA, BIOLOGICAL_MIN_DENSITY_KG_M3,
    BiologicalMaterial, CreatureBodySnapshot, CreatureGenome, CreatureReport, CreatureSimulator,
    CreatureSnapshot, JointGene, MUSCLE_DENSITY_KG_M3, SegmentGene,
};
pub use cuda_probe::{CudaProbeBatchReport, discover_cuda_devices, run_cuda_probe_batch};
pub use evolution::{
    GenomeRng, MutationConfig, MutationKind, MutationRecord, MutationResult,
    crossover_brain_subtree, mutate_genome, random_creature,
};
pub use experiment::{EXPERIMENT_FORMAT_VERSION, ExperimentFile};
pub use fitness::{FitnessConfig, FitnessMetrics, FitnessResult, FitnessWeights, evaluate_fitness};
pub use ga::{
    EvaluatedCreature, EvolutionConfig, EvolutionResult, GenerationSummary, evolve_population,
    evolve_population_checkpointed,
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
