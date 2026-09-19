use std::fs;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};
use evolab_core::{
    AcceleratorConfig, AcceleratorMode, BatchRunner, CreatureGenome, CreatureSimulator,
    CreatureSnapshot, EvolutionCheckpoint, EvolutionConfig, EvolutionResultsFile, ExperimentFile,
    FitnessConfig, FitnessWeights, MutationConfig, PhysicsBackend, ProbeSpec, RapierCpuBackend,
    SimulationConfig, ThroughputMode, TimelineConfig, TrialAggregation, WorldConfig, WorldSnapshot,
    discover_cuda_devices, evolve_population_checkpointed, mutate_genome, random_creature,
    run_cuda_probe_batch,
};
use serde_json::{Value, json};

/// Stream simulation snapshots ahead of the viewer so slow-motion playback can
/// interpolate smoothly instead of waiting on wall-clock-spaced packets.
const STREAM_TRANSPORT_SPEED: f64 = 4.0;

#[derive(Debug, Parser)]
#[command(
    name = "evolab",
    version,
    about = "Headless runner for Modern 3D Creature Evolution"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)] // Parsed once at startup; keep Clap's subcommand fields direct.
enum Command {
    /// Evaluate many independent rigid-body worlds in parallel.
    Probe {
        /// Number of independent worlds to evaluate.
        #[arg(long, default_value_t = 100)]
        batch: usize,

        /// Number of CPU worker threads. 0 lets Rayon choose.
        #[arg(long, default_value_t = 0)]
        workers: usize,

        /// Simulated seconds per world.
        #[arg(long, default_value_t = 5.0)]
        seconds: f32,

        /// Fixed physics timestep in seconds.
        #[arg(long, default_value_t = 1.0 / 120.0)]
        dt: f32,

        /// Serialized WorldConfig JSON.
        #[arg(long)]
        world_json: Option<String>,

        /// Physics backend: cpu, cuda, or auto.
        #[arg(long, default_value = "cpu")]
        backend: String,

        /// CUDA device IDs, for example 0 or 0,1. Empty means all CUDA GPUs.
        #[arg(long, default_value = "")]
        gpus: String,

        /// Emit one machine-readable JSON result to stdout.
        #[arg(long, default_value_t = false)]
        json: bool,

        /// Optional local UDP port for progress/completion events.
        #[arg(long)]
        event_port: Option<u16>,

        /// Host used with --event-port.
        #[arg(long, default_value = "127.0.0.1")]
        event_host: String,
    },

    /// Stream one world's state to the GUI for live playback.
    Stream {
        /// UDP port receiving state frames.
        #[arg(long)]
        event_port: u16,

        /// Host receiving state frames.
        #[arg(long, default_value = "127.0.0.1")]
        event_host: String,

        /// Simulated seconds.
        #[arg(long, default_value_t = 5.0)]
        seconds: f32,

        /// Fixed physics timestep in seconds.
        #[arg(long, default_value_t = 1.0 / 120.0)]
        dt: f32,

        /// Viewer update frequency.
        #[arg(long, default_value_t = 60.0)]
        frame_hz: f32,

        /// Disable real-time pacing and stream as fast as the CPU can simulate.
        #[arg(long, default_value_t = false)]
        max_speed: bool,

        /// Playback speed for live viewing. 1.0 = real time.
        #[arg(long, default_value_t = 1.0)]
        playback_speed: f32,

        /// Serialized WorldConfig JSON.
        #[arg(long)]
        world_json: Option<String>,
    },

    /// Stream the default three-segment creature with motorized joints.
    CreatureStream {
        /// UDP port receiving creature frames.
        #[arg(long)]
        event_port: u16,

        /// Host receiving creature frames.
        #[arg(long, default_value = "127.0.0.1")]
        event_host: String,

        /// Simulated seconds.
        #[arg(long, default_value_t = 8.0)]
        seconds: f32,

        /// Fixed physics timestep in seconds.
        #[arg(long, default_value_t = 1.0 / 120.0)]
        dt: f32,

        /// Viewer update frequency.
        #[arg(long, default_value_t = 60.0)]
        frame_hz: f32,

        /// Disable real-time pacing and stream as fast as the CPU can simulate.
        #[arg(long, default_value_t = false)]
        max_speed: bool,

        /// Playback speed for live viewing. 1.0 = real time.
        #[arg(long, default_value_t = 1.0)]
        playback_speed: f32,

        /// Load an exact creature genome from JSON instead of generating one.
        #[arg(long)]
        genome: Option<PathBuf>,

        /// Reproducible seed used for generation/mutation.
        #[arg(long, default_value_t = 1)]
        seed: u64,

        /// Number of mutation operations applied to the built-in seed creature.
        #[arg(long, default_value_t = 0)]
        mutations: usize,

        /// Generate a random creature with this many segments. 0 uses the built-in seed creature.
        #[arg(long, default_value_t = 0)]
        random_segments: usize,

        /// Hard structural limit used by generation/mutation.
        #[arg(long, default_value_t = 12)]
        max_segments: usize,

        /// Serialized WorldConfig JSON.
        #[arg(long)]
        world_json: Option<String>,

        /// Global actuator strength multiplier.
        #[arg(long, default_value_t = 1.0)]
        motor_strength: f32,
    },

    /// Write a generated/mutated creature genome to a JSON file.
    GenomeGenerate {
        #[arg(long)]
        output: PathBuf,

        #[arg(long, default_value_t = 1)]
        seed: u64,

        /// 0 starts from the built-in three-segment seed; >0 creates a random topology.
        #[arg(long, default_value_t = 0)]
        random_segments: usize,

        #[arg(long, default_value_t = 0)]
        mutations: usize,

        #[arg(long, default_value_t = 12)]
        max_segments: usize,
    },

    /// Evolve a population for distance traveled.
    Evolve {
        /// Optional ancestor genome JSON. Defaults to the built-in three-segment creature.
        #[arg(long)]
        genome: Option<PathBuf>,

        #[arg(long, default_value_t = 50)]
        population: usize,

        #[arg(long, default_value_t = 100)]
        generations: usize,

        #[arg(long, default_value_t = 7)]
        tournament: usize,

        #[arg(long, default_value_t = 2)]
        elite: usize,

        /// Probability that a child receives a donor brain subtree before mutation.
        #[arg(long, default_value_t = 0.5)]
        crossover: f32,

        #[arg(long, default_value_t = 8)]
        mutations: usize,

        /// Chance that a mutation operation attempts a structural body change.
        #[arg(long, default_value_t = 0.30)]
        structural_mutation_chance: f32,

        #[arg(long, default_value_t = 12)]
        max_segments: usize,

        #[arg(long, default_value_t = 1)]
        seed: u64,

        #[arg(long, default_value_t = 0)]
        workers: usize,

        #[arg(long, default_value_t = 5.0)]
        seconds: f32,

        #[arg(long, default_value_t = 1.0 / 120.0)]
        dt: f32,

        /// Serialized WorldConfig JSON.
        #[arg(long)]
        world_json: Option<String>,

        /// Global actuator strength multiplier.
        #[arg(long, default_value_t = 1.0)]
        motor_strength: f32,

        /// Number of deterministic trial seeds per creature.
        #[arg(long, default_value_t = 1)]
        trials: usize,

        /// Trial aggregation: mean, median, worst, or best.
        #[arg(long, default_value = "mean")]
        trial_aggregation: String,

        /// Serialized TimelineConfig JSON.
        #[arg(long)]
        timeline_json: Option<String>,

        /// Weight for horizontal distance traveled.
        #[arg(long, default_value_t = 1.0)]
        fitness_distance: f32,

        /// Weight for average horizontal speed.
        #[arg(long, default_value_t = 0.0)]
        fitness_speed: f32,

        /// Weight for average upright posture.
        #[arg(long, default_value_t = 0.0)]
        fitness_upright: f32,

        /// Weight for low root angular velocity / stability.
        #[arg(long, default_value_t = 0.0)]
        fitness_stability: f32,

        /// Weight for actuator effort. Use a negative value to penalize energy use.
        #[arg(long, default_value_t = 0.0)]
        fitness_energy: f32,

        /// Evolution accelerator policy: cpu, auto, or cuda.
        #[arg(long, default_value = "cpu")]
        accelerator: String,

        /// CUDA device IDs for accelerator scheduling, for example 0 or 0,1.
        #[arg(long, default_value = "")]
        gpus: String,

        /// Preferred accelerator batch size.
        #[arg(long, default_value_t = 4096)]
        gpu_batch_size: usize,

        /// Maximum body parts supported by the planned creature GPU batch layout.
        #[arg(long, default_value_t = 64)]
        gpu_max_parts: usize,

        /// Maximum joints supported by the planned creature GPU batch layout.
        #[arg(long, default_value_t = 128)]
        gpu_max_joints: usize,

        /// Disable explicit CPU fallback when a requested accelerator path is unavailable.
        #[arg(long, default_value_t = false)]
        no_cpu_fallback: bool,

        /// Throughput policy: deterministic or max.
        #[arg(long, default_value = "deterministic")]
        throughput_mode: String,

        /// Write a resumable checkpoint after every completed generation.
        #[arg(long)]
        checkpoint_output: Option<PathBuf>,

        /// Resume an evolution from a generation-boundary checkpoint.
        #[arg(long)]
        resume_checkpoint: Option<PathBuf>,

        /// Optional local UDP port for generation progress/champion events.
        #[arg(long)]
        event_port: Option<u16>,

        #[arg(long, default_value = "127.0.0.1")]
        event_host: String,

        /// Optional path where the current/final champion genome is written.
        #[arg(long)]
        champion_output: Option<PathBuf>,

        /// Optional path where the complete persistent evolution analysis result is written.
        #[arg(long)]
        result_output: Option<PathBuf>,

        /// Human-readable experiment name stored in persisted results.
        #[arg(long, default_value = "Evolution")]
        experiment_name: String,

        /// Emit final evolution result as JSON.
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// Run a complete saved .evo experiment headlessly.
    Run {
        /// Path to a saved EvoLab experiment file.
        experiment: PathBuf,

        /// Override CPU worker threads. Omit to use the experiment value.
        #[arg(long)]
        workers: Option<usize>,

        /// Optional path for the final champion genome JSON.
        #[arg(long)]
        champion_output: Option<PathBuf>,

        /// Optional path for the complete persistent evolution analysis result.
        #[arg(long)]
        result_output: Option<PathBuf>,

        /// Write a resumable checkpoint after every completed generation.
        #[arg(long)]
        checkpoint_output: Option<PathBuf>,

        /// Resume from a checkpoint created from this experiment.
        #[arg(long)]
        resume_checkpoint: Option<PathBuf>,

        /// Emit the complete result as JSON.
        #[arg(long, default_value_t = false)]
        json: bool,
    },

    /// Generate the exact static world geometry used by the simulator.
    WorldGeometry {
        /// Serialized WorldConfig JSON.
        #[arg(long)]
        world_json: Option<String>,
    },

    /// Report backend and machine capabilities.
    Capabilities {
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Command::Probe {
            batch,
            workers,
            seconds,
            dt,
            world_json,
            backend,
            gpus,
            json: json_output,
            event_port,
            event_host,
        } => run_batch(BatchRequest {
            batch,
            workers,
            seconds,
            dt,
            world_json: world_json.as_deref(),
            backend: &backend,
            gpu_ids: parse_gpu_ids(&gpus)?,
            json_output,
            event_host: &event_host,
            event_port,
        }),
        Command::Stream {
            event_port,
            event_host,
            seconds,
            dt,
            frame_hz,
            max_speed,
            playback_speed,
            world_json,
        } => run_stream(StreamRequest {
            event_host: &event_host,
            event_port,
            seconds,
            dt,
            frame_hz,
            realtime: !max_speed,
            playback_speed,
            world_json: world_json.as_deref(),
        }),
        Command::CreatureStream {
            event_port,
            event_host,
            seconds,
            dt,
            frame_hz,
            max_speed,
            playback_speed,
            genome,
            seed,
            mutations,
            random_segments,
            max_segments,
            world_json,
            motor_strength,
        } => run_creature_stream(CreatureStreamRequest {
            event_host: &event_host,
            event_port,
            seconds,
            dt,
            frame_hz,
            realtime: !max_speed,
            playback_speed,
            genome_path: genome.as_ref(),
            seed,
            mutations,
            random_segments,
            max_segments,
            world_json: world_json.as_deref(),
            motor_strength,
        }),
        Command::GenomeGenerate {
            output,
            seed,
            random_segments,
            mutations,
            max_segments,
        } => run_genome_generate(&output, seed, random_segments, mutations, max_segments),
        Command::Evolve {
            genome,
            population,
            generations,
            tournament,
            elite,
            crossover,
            mutations,
            structural_mutation_chance,
            max_segments,
            seed,
            workers,
            seconds,
            dt,
            world_json,
            motor_strength,
            trials,
            trial_aggregation,
            timeline_json,
            fitness_distance,
            fitness_speed,
            fitness_upright,
            fitness_stability,
            fitness_energy,
            accelerator,
            gpus,
            gpu_batch_size,
            gpu_max_parts,
            gpu_max_joints,
            no_cpu_fallback,
            throughput_mode,
            checkpoint_output,
            resume_checkpoint,
            event_port,
            event_host,
            champion_output,
            result_output,
            experiment_name,
            json,
        } => run_evolve(EvolveRequest {
            genome_path: genome.as_ref(),
            population,
            generations,
            tournament,
            elite,
            crossover,
            mutations,
            structural_mutation_chance,
            max_segments,
            seed,
            workers,
            seconds,
            dt,
            world_json: world_json.as_deref(),
            motor_strength,
            trials,
            trial_aggregation: &trial_aggregation,
            timeline_json: timeline_json.as_deref(),
            fitness_distance,
            fitness_speed,
            fitness_upright,
            fitness_stability,
            fitness_energy,
            accelerator: &accelerator,
            gpu_ids: parse_gpu_ids(&gpus)?,
            gpu_batch_size,
            gpu_max_parts,
            gpu_max_joints,
            cpu_fallback: !no_cpu_fallback,
            throughput_mode: &throughput_mode,
            checkpoint_output: checkpoint_output.as_ref(),
            resume_checkpoint: resume_checkpoint.as_ref(),
            event_port,
            event_host: &event_host,
            champion_output: champion_output.as_ref(),
            result_output: result_output.as_ref(),
            experiment_name: &experiment_name,
            json_output: json,
        }),
        Command::Run {
            experiment,
            workers,
            champion_output,
            result_output,
            checkpoint_output,
            resume_checkpoint,
            json,
        } => run_experiment(
            &experiment,
            workers,
            champion_output.as_ref(),
            result_output.as_ref(),
            checkpoint_output.as_ref(),
            resume_checkpoint.as_ref(),
            json,
        ),
        Command::WorldGeometry { world_json } => run_world_geometry(world_json.as_deref()),
        Command::Capabilities { json: json_output } => run_capabilities(json_output),
    }
}

struct BatchRequest<'a> {
    batch: usize,
    workers: usize,
    seconds: f32,
    dt: f32,
    world_json: Option<&'a str>,
    backend: &'a str,
    gpu_ids: Vec<u32>,
    json_output: bool,
    event_host: &'a str,
    event_port: Option<u16>,
}

fn run_batch(request: BatchRequest<'_>) -> Result<(), String> {
    let BatchRequest {
        batch,
        workers,
        seconds,
        dt,
        world_json,
        backend,
        gpu_ids,
        json_output,
        event_host,
        event_port,
    } = request;

    if batch == 0 {
        return Err("batch size must be greater than 0".into());
    }

    let config = simulation_config(seconds, dt, world_json)?;
    let backend_name = backend.trim().to_ascii_lowercase();
    let event_socket = make_event_socket(event_host, event_port)?;

    if backend_name == "cuda" || backend_name == "auto" {
        let cuda_devices = discover_cuda_devices().unwrap_or_default();
        if !cuda_devices.is_empty() {
            let report = run_cuda_probe_batch(&config, &ProbeSpec::default(), batch, &gpu_ids)?;
            let result = json!({
                "protocol_version": 1,
                "kind": "probe_result",
                "backend": report.backend,
                "worlds_evaluated": report.worlds_evaluated,
                "workers_requested": workers,
                "gpu_ids": gpu_ids,
                "physics_dt_seconds": config.dt,
                "steps_per_world": report.steps_per_world,
                "simulated_seconds_per_world": report.simulated_seconds_per_world,
                "wall_seconds": report.wall_seconds,
                "worlds_per_second": report.worlds_per_second,
                "physics_steps_per_second": report.physics_steps_per_second,
                "final_probe_position": report.final_position,
                "final_probe_velocity": report.final_linear_velocity,
                "device_performance": report.devices,
                "deterministic": true,
            });

            if let Some(socket) = event_socket.as_ref() {
                send_event(
                    socket,
                    &json!({
                        "protocol_version": 1,
                        "kind": "batch_started",
                        "total": batch,
                        "backend": result["backend"],
                        "world": config.world,
                        "world_geometry": config.world.geometry(),
                    }),
                );
                send_event(socket, &result);
            }

            if json_output {
                println!("{result}");
            } else if event_socket.is_none() {
                println!("backend              : {}", report.backend);
                println!("worlds evaluated     : {}", report.worlds_evaluated);
                println!("CUDA devices         : {:?}", gpu_ids);
                println!("physics dt           : {:.8} s", config.dt);
                println!("steps/world          : {}", report.steps_per_world);
                println!("wall time            : {:.3} s", report.wall_seconds);
                println!(
                    "throughput           : {:.1} worlds/s",
                    report.worlds_per_second
                );
                println!(
                    "physics throughput   : {:.0} steps/s",
                    report.physics_steps_per_second
                );
            }
            return Ok(());
        } else if backend_name == "cuda" {
            return Err("CUDA backend requested, but no CUDA device is available".into());
        }
    } else if backend_name != "cpu" {
        return Err("backend must be cpu, cuda, or auto".into());
    }

    let runner = BatchRunner { threads: workers };
    let backend = RapierCpuBackend;

    if let Some(socket) = event_socket.as_ref() {
        send_event(
            socket,
            &json!({
                "protocol_version": 1,
                "kind": "batch_started",
                "total": batch,
                "backend": backend.name(),
                "world": config.world,
                "world_geometry": config.world.geometry(),
            }),
        );
    }

    let progress_interval = (batch / 100).max(1);
    let started = Instant::now();

    let reports = if let Some(socket) = event_socket.as_ref() {
        runner.run_identical_with_progress(
            &backend,
            &config,
            ProbeSpec::default(),
            batch,
            |completed, total| {
                if completed == total || completed % progress_interval == 0 {
                    send_event(
                        socket,
                        &json!({
                            "protocol_version": 1,
                            "kind": "batch_progress",
                            "completed": completed,
                            "total": total,
                            "fraction": completed as f64 / total as f64,
                        }),
                    );
                }
            },
        )?
    } else {
        runner.run_identical(&backend, &config, ProbeSpec::default(), batch)?
    };

    let elapsed = started.elapsed();
    let first = &reports[0];
    let worlds_per_second = batch as f64 / elapsed.as_secs_f64();
    let physics_steps = reports.len() as u64 * first.steps as u64;
    let physics_steps_per_second = physics_steps as f64 / elapsed.as_secs_f64();

    let result = json!({
        "protocol_version": 1,
        "kind": "probe_result",
        "backend": first.backend,
        "worlds_evaluated": reports.len(),
        "workers_requested": workers,
        "physics_dt_seconds": config.dt,
        "steps_per_world": first.steps,
        "simulated_seconds_per_world": first.simulated_seconds,
        "wall_seconds": elapsed.as_secs_f64(),
        "worlds_per_second": worlds_per_second,
        "physics_steps_per_second": physics_steps_per_second,
        "final_probe_position": first.final_position,
        "final_probe_velocity": first.final_linear_velocity,
        "deterministic": config.deterministic,
    });

    if let Some(socket) = event_socket.as_ref() {
        send_event(socket, &result);
    }

    if json_output {
        println!("{result}");
    } else if event_socket.is_none() {
        println!("backend              : {}", first.backend);
        println!("worlds evaluated     : {}", reports.len());
        println!("workers requested    : {workers}");
        println!("physics dt           : {:.8} s", config.dt);
        println!("steps/world          : {}", first.steps);
        println!("simulated time/world : {:.3} s", first.simulated_seconds);
        println!("wall time            : {:.3} s", elapsed.as_secs_f64());
        println!("throughput           : {:.1} worlds/s", worlds_per_second);
        println!(
            "physics throughput   : {:.0} steps/s",
            physics_steps_per_second
        );
        println!(
            "final probe position  : [{:.4}, {:.4}, {:.4}]",
            first.final_position[0], first.final_position[1], first.final_position[2]
        );
    }

    Ok(())
}

struct StreamRequest<'a> {
    event_host: &'a str,
    event_port: u16,
    seconds: f32,
    dt: f32,
    frame_hz: f32,
    realtime: bool,
    playback_speed: f32,
    world_json: Option<&'a str>,
}

fn run_stream(request: StreamRequest<'_>) -> Result<(), String> {
    let StreamRequest {
        event_host,
        event_port,
        seconds,
        dt,
        frame_hz,
        realtime,
        playback_speed,
        world_json,
    } = request;

    if !frame_hz.is_finite() || !(1.0..=240.0).contains(&frame_hz) {
        return Err("frame_hz must be between 1 and 240".into());
    }
    if !playback_speed.is_finite() || !(0.01..=2.0).contains(&playback_speed) {
        return Err("playback_speed must be between 0.01 and 2.0".into());
    }

    let config = simulation_config(seconds, dt, world_json)?;

    let socket = make_event_socket(event_host, Some(event_port))?
        .ok_or_else(|| "streaming requires an event port".to_string())?;
    let backend = RapierCpuBackend;
    let sample_every_steps = ((1.0 / frame_hz) / config.dt).round().max(1.0) as usize;

    send_event(
        &socket,
        &json!({
            "protocol_version": 1,
            "kind": "stream_started",
            "backend": backend.name(),
            "frame_hz": frame_hz,
            "sample_every_steps": sample_every_steps,
            "realtime": realtime,
            "playback_speed": playback_speed,
            "world": config.world,
            "world_geometry": config.world.geometry(),
        }),
    );

    let wall_start = Instant::now();
    let mut observer = |snapshot: &WorldSnapshot| -> Result<(), String> {
        if realtime {
            let target = wall_start
                + Duration::from_secs_f64(
                    snapshot.simulated_seconds as f64 / STREAM_TRANSPORT_SPEED,
                );
            let now = Instant::now();
            if target > now {
                thread::sleep(target - now);
            }
        }

        send_event(
            &socket,
            &json!({
                "protocol_version": 1,
                "kind": "world_state",
                "state": snapshot,
            }),
        );
        Ok(())
    };

    let report = backend.run_probe_streaming(
        &config,
        &ProbeSpec::default(),
        sample_every_steps,
        &mut observer,
    )?;

    send_event(
        &socket,
        &json!({
            "protocol_version": 1,
            "kind": "stream_complete",
            "backend": report.backend,
            "steps": report.steps,
            "simulated_seconds": report.simulated_seconds,
            "final_probe_position": report.final_position,
            "final_probe_velocity": report.final_linear_velocity,
        }),
    );

    Ok(())
}

struct CreatureStreamRequest<'a> {
    event_host: &'a str,
    event_port: u16,
    seconds: f32,
    dt: f32,
    frame_hz: f32,
    realtime: bool,
    playback_speed: f32,
    genome_path: Option<&'a PathBuf>,
    seed: u64,
    mutations: usize,
    random_segments: usize,
    max_segments: usize,
    world_json: Option<&'a str>,
    motor_strength: f32,
}

fn run_creature_stream(request: CreatureStreamRequest<'_>) -> Result<(), String> {
    let CreatureStreamRequest {
        event_host,
        event_port,
        seconds,
        dt,
        frame_hz,
        realtime,
        playback_speed,
        genome_path,
        seed,
        mutations,
        random_segments,
        max_segments,
        world_json,
        motor_strength,
    } = request;
    if !frame_hz.is_finite() || !(1.0..=240.0).contains(&frame_hz) {
        return Err("frame_hz must be between 1 and 240".into());
    }
    if !playback_speed.is_finite() || !(0.01..=2.0).contains(&playback_speed) {
        return Err("playback_speed must be between 0.01 and 2.0".into());
    }

    let mut config = simulation_config(seconds, dt, world_json)?;
    config.motor_strength_multiplier = motor_strength;
    config.validate()?;

    let socket = make_event_socket(event_host, Some(event_port))?
        .ok_or_else(|| "creature streaming requires an event port".to_string())?;
    let simulator = CreatureSimulator;
    let mutation_config = MutationConfig {
        max_segments: max_segments.max(2),
        ..MutationConfig::default()
    };

    let (base_genome, mut mutation_log, mut genome_source) = if let Some(path) = genome_path {
        let raw = fs::read_to_string(path)
            .map_err(|err| format!("failed to read genome {}: {err}", path.display()))?;
        let genome: CreatureGenome = serde_json::from_str(&raw)
            .map_err(|err| format!("invalid genome JSON {}: {err}", path.display()))?;
        genome.validate()?;
        (genome, Vec::new(), format!("file:{}", path.display()))
    } else if random_segments > 0 {
        let result = random_creature(seed, random_segments, &mutation_config)?;
        (result.genome, result.mutations, format!("random:{seed}"))
    } else {
        (
            CreatureGenome::three_segment_walker(),
            Vec::new(),
            "built-in".to_string(),
        )
    };

    let genome = if mutations > 0 {
        let result = mutate_genome(
            &base_genome,
            seed ^ 0xD1B5_4A32_D192_ED03,
            mutations,
            &mutation_config,
        )?;
        mutation_log.extend(result.mutations);
        genome_source = format!("{genome_source}+mutated:{mutations}");
        result.genome
    } else {
        base_genome
    };

    let sample_every_steps = ((1.0 / frame_hz) / config.dt).round().max(1.0) as usize;

    send_event(
        &socket,
        &json!({
            "protocol_version": 1,
            "kind": "creature_stream_started",
            "creature_name": genome.name,
            "segment_count": genome.segments.len(),
            "joint_count": genome.joints.len(),
            "brain_outputs": genome.brain.outputs.len(),
            "brain_nodes": genome.brain.node_count(),
            "brain_sensor_nodes": genome.brain.sensor_node_count(),
            "brain_unique_sensors": genome.brain.unique_sensor_count(),
            "frame_hz": frame_hz,
            "sample_every_steps": sample_every_steps,
            "realtime": realtime,
            "playback_speed": playback_speed,
            "genome_source": genome_source,
            "mutation_log": mutation_log,
            "genome": genome,
            "world": config.world,
            "world_geometry": config.world.geometry(),
            "motor_strength_multiplier": config.motor_strength_multiplier,
        }),
    );

    let wall_start = Instant::now();
    let mut observer = |snapshot: &CreatureSnapshot| -> Result<(), String> {
        if realtime {
            let target = wall_start
                + Duration::from_secs_f64(
                    snapshot.simulated_seconds as f64 / STREAM_TRANSPORT_SPEED,
                );
            let now = Instant::now();
            if target > now {
                thread::sleep(target - now);
            }
        }

        send_event(
            &socket,
            &json!({
                "protocol_version": 1,
                "kind": "creature_state",
                "state": snapshot,
            }),
        );
        Ok(())
    };

    let report = simulator.run_streaming(&config, &genome, sample_every_steps, &mut observer)?;

    send_event(
        &socket,
        &json!({
            "protocol_version": 1,
            "kind": "creature_stream_complete",
            "steps": report.steps,
            "simulated_seconds": report.simulated_seconds,
            "final_root_position": report.final_root_position,
        }),
    );

    Ok(())
}

fn run_genome_generate(
    output: &PathBuf,
    seed: u64,
    random_segments: usize,
    mutations: usize,
    max_segments: usize,
) -> Result<(), String> {
    let mutation_config = MutationConfig {
        max_segments: max_segments.max(2),
        ..MutationConfig::default()
    };

    let genome = if random_segments > 0 {
        random_creature(seed, random_segments, &mutation_config)?.genome
    } else if mutations > 0 {
        mutate_genome(
            &CreatureGenome::three_segment_walker(),
            seed,
            mutations,
            &mutation_config,
        )?
        .genome
    } else {
        CreatureGenome::three_segment_walker()
    };

    genome.validate()?;
    let json = serde_json::to_string_pretty(&genome)
        .map_err(|err| format!("failed to serialize genome: {err}"))?;
    fs::write(output, json)
        .map_err(|err| format!("failed to write genome {}: {err}", output.display()))?;
    println!("wrote {}", output.display());
    Ok(())
}

struct EvolveRequest<'a> {
    genome_path: Option<&'a PathBuf>,
    population: usize,
    generations: usize,
    tournament: usize,
    elite: usize,
    crossover: f32,
    mutations: usize,
    structural_mutation_chance: f32,
    max_segments: usize,
    seed: u64,
    workers: usize,
    seconds: f32,
    dt: f32,
    world_json: Option<&'a str>,
    motor_strength: f32,
    trials: usize,
    trial_aggregation: &'a str,
    timeline_json: Option<&'a str>,
    fitness_distance: f32,
    fitness_speed: f32,
    fitness_upright: f32,
    fitness_stability: f32,
    fitness_energy: f32,
    accelerator: &'a str,
    gpu_ids: Vec<u32>,
    gpu_batch_size: usize,
    gpu_max_parts: usize,
    gpu_max_joints: usize,
    cpu_fallback: bool,
    throughput_mode: &'a str,
    checkpoint_output: Option<&'a PathBuf>,
    resume_checkpoint: Option<&'a PathBuf>,
    event_port: Option<u16>,
    event_host: &'a str,
    champion_output: Option<&'a PathBuf>,
    result_output: Option<&'a PathBuf>,
    experiment_name: &'a str,
    json_output: bool,
}

fn run_evolve(request: EvolveRequest<'_>) -> Result<(), String> {
    let socket = make_event_socket(request.event_host, request.event_port)?;
    let result = run_evolve_inner(&request, socket.as_ref());

    if let Err(err) = &result
        && let Some(socket) = socket.as_ref()
    {
        send_event(
            socket,
            &json!({
                "protocol_version": 1,
                "kind": "evolution_error",
                "message": err,
            }),
        );
    }

    result
}

fn run_evolve_inner(request: &EvolveRequest<'_>, socket: Option<&UdpSocket>) -> Result<(), String> {
    let ancestor = if let Some(path) = request.genome_path {
        let raw = fs::read_to_string(path)
            .map_err(|err| format!("failed to read genome {}: {err}", path.display()))?;
        let mut genome: CreatureGenome = serde_json::from_str(&raw)
            .map_err(|err| format!("invalid genome JSON {}: {err}", path.display()))?;
        genome
            .brain
            .sync_with_structure(&genome.joints, &genome.segments);
        genome.validate()?;
        genome
    } else {
        CreatureGenome::three_segment_walker()
    };

    let mut simulation = simulation_config(request.seconds, request.dt, request.world_json)?;
    simulation.motor_strength_multiplier = request.motor_strength;

    let timeline = if let Some(raw) = request.timeline_json {
        serde_json::from_str::<TimelineConfig>(raw)
            .map_err(|err| format!("invalid --timeline-json: {err}"))?
    } else {
        TimelineConfig::default()
    };

    let trial_aggregation = parse_trial_aggregation(request.trial_aggregation)?;

    let config = EvolutionConfig {
        population_size: request.population,
        generations: request.generations,
        tournament_size: request.tournament,
        elite_count: request.elite,
        crossover_chance: request.crossover,
        mutations_per_child: request.mutations,
        seed: request.seed,
        worker_threads: request.workers,
        simulation,
        fitness: FitnessConfig {
            weights: FitnessWeights {
                distance: request.fitness_distance,
                average_speed: request.fitness_speed,
                upright: request.fitness_upright,
                stability: request.fitness_stability,
                energy: request.fitness_energy,
            },
        },
        mutation: MutationConfig {
            max_segments: request.max_segments.max(2),
            structural_mutation_chance: request.structural_mutation_chance,
            ..MutationConfig::default()
        },
        trials_per_creature: request.trials,
        trial_aggregation,
        accelerator: AcceleratorConfig {
            mode: parse_accelerator_mode(request.accelerator)?,
            gpu_ids: request.gpu_ids.clone(),
            batch_size: request.gpu_batch_size,
            max_parts: request.gpu_max_parts,
            max_joints: request.gpu_max_joints,
            cpu_fallback: request.cpu_fallback,
            throughput_mode: parse_throughput_mode(request.throughput_mode)?,
        },
        timeline,
    };
    config.validate()?;

    let resume_checkpoint = if let Some(path) = request.resume_checkpoint {
        Some(read_checkpoint(path)?)
    } else {
        None
    };

    if let Some(checkpoint) = resume_checkpoint.as_ref()
        && checkpoint.config != config
    {
        return Err(
            "resume checkpoint configuration does not match the requested evolution settings"
                .into(),
        );
    }

    if let Some(socket) = socket {
        send_event(
            socket,
            &json!({
                "protocol_version": 1,
                "kind": "evolution_started",
                "population": config.population_size,
                "generations": config.generations,
                "tournament": config.tournament_size,
                "elite": config.elite_count,
                "crossover_chance": config.crossover_chance,
                "mutations_per_child": config.mutations_per_child,
                "structural_mutation_chance": config.mutation.structural_mutation_chance,
                "fitness_weights": config.fitness.weights,
                "motor_strength_multiplier": config.simulation.motor_strength_multiplier,
                "trials_per_creature": config.trials_per_creature,
                "trial_aggregation": config.trial_aggregation,
                "timeline": config.timeline,
                "accelerator": config.accelerator,
                "resuming_from_generation": resume_checkpoint
                    .as_ref()
                    .map(|checkpoint| checkpoint.next_generation),
                "world": config.simulation.world,
                "world_geometry": config.simulation.world.geometry(),
            }),
        );
    }

    let started = Instant::now();
    let result = evolve_population_checkpointed(
        &ancestor,
        &config,
        resume_checkpoint.as_ref(),
        |summary| {
            if let Some(path) = request.champion_output {
                write_genome(path, &summary.champion)?;
            }

            if let Some(socket) = socket {
                send_event(
                    socket,
                    &json!({
                        "protocol_version": 1,
                        "kind": "generation_complete",
                        "generation": summary.generation,
                        "generations": config.generations,
                        "fraction": summary.generation as f64 / config.generations as f64,
                        "best_fitness": summary.best_fitness,
                        "average_fitness": summary.average_fitness,
                        "median_fitness": summary.median_fitness,
                        "worst_fitness": summary.worst_fitness,
                        "best_distance": summary.best_distance,
                        "best_metrics": summary.best_metrics,
                        "best_segments": summary.best_segments,
                        "best_joints": summary.best_joints,
                        "best_brain_nodes": summary.best_brain_nodes,
                        "best_brain_sensor_nodes": summary.best_brain_sensor_nodes,
                        "best_brain_unique_sensors": summary.best_brain_unique_sensors,
                        "best_brain_outputs": summary.best_brain_outputs,
                        "champion_id": summary.champion_id,
                        "champion_parent_ids": summary.champion_parent_ids,
                        "champion_species_id": summary.champion_species_id,
                        "diversity": summary.diversity,
                        "analysis_species_count": summary.species.len(),
                        "pareto_front_size": summary.pareto_front.len(),
                        "map_elites_cells": summary.map_elites.len(),
                        "evaluations_completed": summary.evaluations_completed,
                        "execution": summary.execution,
                        "effective_population": summary.effective_settings.population_size,
                        "effective_mutations_per_child": summary.effective_settings.mutations_per_child,
                        "effective_structural_mutation_chance":
                            summary.effective_settings.mutation.structural_mutation_chance,
                        "effective_motor_strength_multiplier":
                            summary.effective_settings.simulation.motor_strength_multiplier,
                        "effective_duration_seconds":
                            summary.effective_settings.simulation.duration_seconds,
                        "effective_trials_per_creature":
                            summary.effective_settings.trials_per_creature,
                        "effective_trial_aggregation":
                            summary.effective_settings.trial_aggregation,
                        "effective_fitness_weights": summary.effective_settings.fitness.weights,
                        "effective_world": summary.effective_settings.simulation.world,
                        "active_timeline_events": summary.active_timeline_events,
                        "triggered_timeline_events": summary.triggered_timeline_events,
                        "champion_file": request
                            .champion_output
                            .map(|path| path.to_string_lossy().to_string()),
                    }),
                );
            } else {
                println!(
                    "generation {:>4}/{:<4}  best {:>8.4}  avg {:>8.4}  distance {:>8.4} m  segments {}  brain {}",
                    summary.generation,
                    config.generations,
                    summary.best_fitness,
                    summary.average_fitness,
                    summary.best_distance,
                    summary.best_segments,
                    summary.best_brain_nodes
                );
            }
            Ok(())
        },
        |checkpoint| {
            if let Some(path) = request.checkpoint_output {
                write_checkpoint(path, checkpoint)?;
            }
            Ok(())
        },
    )?;

    let elapsed = started.elapsed();

    if let Some(path) = request.champion_output {
        write_genome(path, &result.champion)?;
    }
    if let Some(path) = request.result_output {
        write_results_file(
            path,
            request.experiment_name,
            &config,
            elapsed.as_secs_f64(),
            &result,
        )?;
    }

    let transport_event = json!({
        "protocol_version": 1,
        "kind": "evolution_complete",
        "champion_fitness": result.champion_fitness,
        "champion_distance": result.champion_distance,
        "champion_metrics": result.champion_metrics,
        "fitness_weights": config.fitness.weights,
        "champion_segments": result.champion.segments.len(),
        "champion_joints": result.champion.joints.len(),
        "champion_brain_nodes": result.champion.brain.node_count(),
        "champion_brain_sensor_nodes": result.champion.brain.sensor_node_count(),
        "champion_brain_unique_sensors": result.champion.brain.unique_sensor_count(),
        "champion_brain_outputs": result.champion.brain.outputs.len(),
        "generations_completed": result.generations_completed,
        "evaluations_completed": result.evaluations_completed,
        "final_settings": result.final_settings,
        "wall_seconds": elapsed.as_secs_f64(),
        "champion_file": request
            .champion_output
            .map(|path| path.to_string_lossy().to_string()),
        "results_file": request
            .result_output
            .map(|path| path.to_string_lossy().to_string()),
        "checkpoint_file": request
            .checkpoint_output
            .map(|path| path.to_string_lossy().to_string()),
    });

    if let Some(socket) = socket {
        send_event(socket, &transport_event);
    }

    if request.json_output {
        println!(
            "{}",
            json!({
                "protocol_version": 1,
                "kind": "evolution_complete",
                "champion_fitness": result.champion_fitness,
                "champion_distance": result.champion_distance,
                "champion_metrics": result.champion_metrics,
                "fitness_weights": config.fitness.weights,
                "generations_completed": result.generations_completed,
                "evaluations_completed": result.evaluations_completed,
                "final_settings": result.final_settings,
                "wall_seconds": elapsed.as_secs_f64(),
                "champion": result.champion,
                "history": result.history,
            })
        );
    } else if socket.is_none() {
        println!(
            "complete: champion score {:.4} (distance {:.4} m) after {} generations / {} evaluations in {:.3} s",
            result.champion_fitness,
            result.champion_distance,
            result.generations_completed,
            result.evaluations_completed,
            elapsed.as_secs_f64()
        );
    }

    Ok(())
}

fn run_experiment(
    path: &PathBuf,
    workers: Option<usize>,
    champion_output: Option<&PathBuf>,
    result_output: Option<&PathBuf>,
    checkpoint_output: Option<&PathBuf>,
    resume_checkpoint: Option<&PathBuf>,
    json_output: bool,
) -> Result<(), String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("failed to read experiment {}: {err}", path.display()))?;
    let mut experiment: ExperimentFile = serde_json::from_str(&raw)
        .map_err(|err| format!("invalid experiment JSON {}: {err}", path.display()))?;
    experiment.validate()?;

    if let Some(worker_threads) = workers {
        experiment.evolution.worker_threads = worker_threads;
    }

    let checkpoint = if let Some(path) = resume_checkpoint {
        Some(read_checkpoint(path)?)
    } else {
        None
    };
    if let Some(checkpoint) = checkpoint.as_ref()
        && checkpoint.config != experiment.evolution
    {
        return Err("resume checkpoint does not match the experiment configuration".into());
    }

    let started = Instant::now();
    let result = evolve_population_checkpointed(
        &experiment.ancestor,
        &experiment.evolution,
        checkpoint.as_ref(),
        |summary| {
            if !json_output {
                println!(
                    "generation {:>4}/{:<4}  best {:>8.4}  avg {:>8.4}  distance {:>8.4} m  trials {}",
                    summary.generation,
                    experiment.evolution.generations,
                    summary.best_fitness,
                    summary.average_fitness,
                    summary.best_distance,
                    summary.effective_settings.trials_per_creature,
                );
                if !summary.triggered_timeline_events.is_empty() {
                    println!(
                        "  timeline triggered: {}",
                        summary.triggered_timeline_events.join(", ")
                    );
                }
            }
            Ok(())
        },
        |checkpoint| {
            if let Some(path) = checkpoint_output {
                write_checkpoint(path, checkpoint)?;
            }
            Ok(())
        },
    )?;
    let elapsed = started.elapsed();

    if let Some(output) = champion_output {
        write_genome(output, &result.champion)?;
    }
    if let Some(output) = result_output {
        write_results_file(
            output,
            &experiment.name,
            &experiment.evolution,
            elapsed.as_secs_f64(),
            &result,
        )?;
    }

    if json_output {
        println!(
            "{}",
            json!({
                "protocol_version": 1,
                "kind": "experiment_complete",
                "experiment_name": experiment.name,
                "champion_fitness": result.champion_fitness,
                "champion_distance": result.champion_distance,
                "champion_metrics": result.champion_metrics,
                "generations_completed": result.generations_completed,
                "evaluations_completed": result.evaluations_completed,
                "final_settings": result.final_settings,
                "wall_seconds": elapsed.as_secs_f64(),
                "champion": result.champion,
                "history": result.history,
            })
        );
    } else {
        println!(
            "complete: '{}' champion {:.4} (distance {:.4} m), {} evaluations in {:.3} s",
            experiment.name,
            result.champion_fitness,
            result.champion_distance,
            result.evaluations_completed,
            elapsed.as_secs_f64(),
        );
    }

    Ok(())
}

fn read_checkpoint(path: &PathBuf) -> Result<EvolutionCheckpoint, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("failed to read checkpoint {}: {err}", path.display()))?;
    let checkpoint: EvolutionCheckpoint = serde_json::from_str(&raw)
        .map_err(|err| format!("invalid checkpoint JSON {}: {err}", path.display()))?;
    checkpoint.validate()?;
    Ok(checkpoint)
}

fn write_checkpoint(path: &PathBuf, checkpoint: &EvolutionCheckpoint) -> Result<(), String> {
    checkpoint.validate()?;
    let json = serde_json::to_string_pretty(checkpoint)
        .map_err(|err| format!("failed to serialize checkpoint: {err}"))?;
    let temp_path = path.with_extension("checkpoint.tmp");
    fs::write(&temp_path, json)
        .map_err(|err| format!("failed to write checkpoint {}: {err}", temp_path.display()))?;
    fs::rename(&temp_path, path)
        .map_err(|err| format!("failed to replace checkpoint {}: {err}", path.display()))
}

fn parse_accelerator_mode(value: &str) -> Result<AcceleratorMode, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "cpu" => Ok(AcceleratorMode::Cpu),
        "auto" => Ok(AcceleratorMode::Auto),
        "cuda" => Ok(AcceleratorMode::Cuda),
        other => Err(format!(
            "invalid accelerator '{other}'; expected cpu, auto, or cuda"
        )),
    }
}

fn parse_throughput_mode(value: &str) -> Result<ThroughputMode, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "deterministic" => Ok(ThroughputMode::Deterministic),
        "max" | "max_throughput" | "max-throughput" => Ok(ThroughputMode::MaxThroughput),
        other => Err(format!(
            "invalid throughput mode '{other}'; expected deterministic or max"
        )),
    }
}

fn write_results_file(
    path: &PathBuf,
    experiment_name: &str,
    evolution: &EvolutionConfig,
    wall_seconds: f64,
    result: &evolab_core::EvolutionResult,
) -> Result<(), String> {
    let results = EvolutionResultsFile::new(
        experiment_name,
        evolution.clone(),
        wall_seconds,
        result.clone(),
    );
    results.validate()?;
    let json = serde_json::to_string_pretty(&results)
        .map_err(|err| format!("failed to serialize evolution results: {err}"))?;
    fs::write(path, json)
        .map_err(|err| format!("failed to write results {}: {err}", path.display()))
}

fn write_genome(path: &PathBuf, genome: &CreatureGenome) -> Result<(), String> {
    let json = serde_json::to_string_pretty(genome)
        .map_err(|err| format!("failed to serialize champion genome: {err}"))?;
    fs::write(path, json)
        .map_err(|err| format!("failed to write champion genome {}: {err}", path.display()))
}

fn parse_trial_aggregation(value: &str) -> Result<TrialAggregation, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "mean" => Ok(TrialAggregation::Mean),
        "median" => Ok(TrialAggregation::Median),
        "worst" => Ok(TrialAggregation::Worst),
        "best" => Ok(TrialAggregation::Best),
        other => Err(format!(
            "invalid trial aggregation '{other}'; expected mean, median, worst, or best"
        )),
    }
}

fn parse_gpu_ids(value: &str) -> Result<Vec<u32>, String> {
    if value.trim().is_empty() {
        return Ok(Vec::new());
    }

    value
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<u32>()
                .map_err(|_| format!("invalid GPU id '{}'", part.trim()))
        })
        .collect()
}

fn simulation_config(
    seconds: f32,
    dt: f32,
    world_json: Option<&str>,
) -> Result<SimulationConfig, String> {
    let world = if let Some(raw) = world_json {
        serde_json::from_str::<WorldConfig>(raw)
            .map_err(|err| format!("invalid --world-json: {err}"))?
    } else {
        WorldConfig::default()
    };

    let config = SimulationConfig {
        duration_seconds: seconds,
        dt,
        world,
        ..SimulationConfig::default()
    };
    config.validate()?;
    Ok(config)
}

fn run_world_geometry(world_json: Option<&str>) -> Result<(), String> {
    let world = if let Some(raw) = world_json {
        serde_json::from_str::<WorldConfig>(raw)
            .map_err(|err| format!("invalid --world-json: {err}"))?
    } else {
        WorldConfig::default()
    };
    world.validate()?;

    println!(
        "{}",
        json!({
            "protocol_version": 1,
            "kind": "world_geometry",
            "world": world,
            "geometry": world.geometry(),
        })
    );
    Ok(())
}

fn run_capabilities(json_output: bool) -> Result<(), String> {
    let backend = RapierCpuBackend;
    let logical_cpu_threads = thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(1);
    let cuda_devices = discover_cuda_devices().unwrap_or_default();

    let mut backends = vec![
        serde_json::to_value(backend.capabilities())
            .map_err(|err| format!("failed to serialize backend capabilities: {err}"))?,
    ];
    if !cuda_devices.is_empty() {
        backends.push(json!({
            "name": "cuda-probe",
            "deterministic": true,
            "state_streaming": false,
            "parallel_worlds": true,
            "gpu_accelerated": true,
            "creature_physics": false,
            "scope": "flat-world probe batches",
        }));
    }

    let result = json!({
        "protocol_version": 1,
        "kind": "capabilities",
        "app_version": env!("CARGO_PKG_VERSION"),
        "logical_cpu_threads": logical_cpu_threads,
        "cuda_devices": cuda_devices,
        "backends": backends,
    });

    if json_output {
        println!("{result}");
    } else {
        println!("Modern 3D Creature Evolution {}", env!("CARGO_PKG_VERSION"));
        println!("logical CPU threads : {logical_cpu_threads}");
        println!("backend             : {}", backend.name());
        println!("state streaming     : yes");
        println!("deterministic       : yes");
        println!("CUDA devices         : {}", cuda_devices.len());
        println!(
            "GPU probe backend    : {}",
            if cuda_devices.is_empty() {
                "unavailable"
            } else {
                "available"
            }
        );
    }

    Ok(())
}

fn make_event_socket(host: &str, port: Option<u16>) -> Result<Option<UdpSocket>, String> {
    let Some(port) = port else {
        return Ok(None);
    };

    let socket = UdpSocket::bind("127.0.0.1:0")
        .map_err(|err| format!("failed to open event socket: {err}"))?;
    socket
        .connect((host, port))
        .map_err(|err| format!("failed to connect event socket to {host}:{port}: {err}"))?;
    Ok(Some(socket))
}

fn send_event(socket: &UdpSocket, value: &Value) {
    if let Ok(payload) = serde_json::to_vec(value) {
        let _ = socket.send(&payload);
    }
}
