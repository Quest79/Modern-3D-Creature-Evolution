use std::fs;
use std::net::UdpSocket;
use std::path::PathBuf;
use std::process::ExitCode;
use std::thread;
use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};
use evolab_core::{
    BatchRunner, CreatureGenome, CreatureSimulator, CreatureSnapshot, EvolutionConfig,
    MutationConfig, PhysicsBackend, ProbeSpec, RapierCpuBackend, SimulationConfig, WorldSnapshot,
    evolve_population, mutate_genome, random_creature,
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

        /// Optional local UDP port for generation progress/champion events.
        #[arg(long)]
        event_port: Option<u16>,

        #[arg(long, default_value = "127.0.0.1")]
        event_host: String,

        /// Optional path where the current/final champion genome is written.
        #[arg(long)]
        champion_output: Option<PathBuf>,

        /// Emit final evolution result as JSON.
        #[arg(long, default_value_t = false)]
        json: bool,
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
            json: json_output,
            event_port,
            event_host,
        } => run_batch(
            batch,
            workers,
            seconds,
            dt,
            json_output,
            &event_host,
            event_port,
        ),
        Command::Stream {
            event_port,
            event_host,
            seconds,
            dt,
            frame_hz,
            max_speed,
            playback_speed,
        } => run_stream(
            &event_host,
            event_port,
            seconds,
            dt,
            frame_hz,
            !max_speed,
            playback_speed,
        ),
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
            max_segments,
            seed,
            workers,
            seconds,
            dt,
            event_port,
            event_host,
            champion_output,
            json,
        } => run_evolve(EvolveRequest {
            genome_path: genome.as_ref(),
            population,
            generations,
            tournament,
            elite,
            crossover,
            mutations,
            max_segments,
            seed,
            workers,
            seconds,
            dt,
            event_port,
            event_host: &event_host,
            champion_output: champion_output.as_ref(),
            json_output: json,
        }),
        Command::Capabilities { json: json_output } => run_capabilities(json_output),
    }
}

fn run_batch(
    batch: usize,
    workers: usize,
    seconds: f32,
    dt: f32,
    json_output: bool,
    event_host: &str,
    event_port: Option<u16>,
) -> Result<(), String> {
    if batch == 0 {
        return Err("batch size must be greater than 0".into());
    }

    let config = SimulationConfig {
        duration_seconds: seconds,
        dt,
        ..SimulationConfig::default()
    };
    config.validate()?;

    let runner = BatchRunner { threads: workers };
    let backend = RapierCpuBackend;
    let event_socket = make_event_socket(event_host, event_port)?;

    if let Some(socket) = event_socket.as_ref() {
        send_event(
            socket,
            &json!({
                "protocol_version": 1,
                "kind": "batch_started",
                "total": batch,
                "backend": backend.name(),
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

fn run_stream(
    event_host: &str,
    event_port: u16,
    seconds: f32,
    dt: f32,
    frame_hz: f32,
    realtime: bool,
    playback_speed: f32,
) -> Result<(), String> {
    if !frame_hz.is_finite() || !(1.0..=240.0).contains(&frame_hz) {
        return Err("frame_hz must be between 1 and 240".into());
    }
    if !playback_speed.is_finite() || !(0.01..=2.0).contains(&playback_speed) {
        return Err("playback_speed must be between 0.01 and 2.0".into());
    }

    let config = SimulationConfig {
        duration_seconds: seconds,
        dt,
        ..SimulationConfig::default()
    };
    config.validate()?;

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
    } = request;
    if !frame_hz.is_finite() || !(1.0..=240.0).contains(&frame_hz) {
        return Err("frame_hz must be between 1 and 240".into());
    }
    if !playback_speed.is_finite() || !(0.01..=2.0).contains(&playback_speed) {
        return Err("playback_speed must be between 0.01 and 2.0".into());
    }

    let config = SimulationConfig {
        duration_seconds: seconds,
        dt,
        ..SimulationConfig::default()
    };
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
    max_segments: usize,
    seed: u64,
    workers: usize,
    seconds: f32,
    dt: f32,
    event_port: Option<u16>,
    event_host: &'a str,
    champion_output: Option<&'a PathBuf>,
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

    let config = EvolutionConfig {
        population_size: request.population,
        generations: request.generations,
        tournament_size: request.tournament,
        elite_count: request.elite,
        crossover_chance: request.crossover,
        mutations_per_child: request.mutations,
        seed: request.seed,
        worker_threads: request.workers,
        simulation: SimulationConfig {
            duration_seconds: request.seconds,
            dt: request.dt,
            ..SimulationConfig::default()
        },
        mutation: MutationConfig {
            max_segments: request.max_segments.max(2),
            ..MutationConfig::default()
        },
    };
    config.validate()?;

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
                "fitness": "horizontal_distance",
            }),
        );
    }

    let started = Instant::now();
    let result = evolve_population(&ancestor, &config, |summary| {
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
                    "worst_fitness": summary.worst_fitness,
                    "best_distance": summary.best_distance,
                    "best_segments": summary.best_segments,
                    "best_joints": summary.best_joints,
                    "best_brain_nodes": summary.best_brain_nodes,
                    "best_brain_sensor_nodes": summary.best_brain_sensor_nodes,
                    "best_brain_unique_sensors": summary.best_brain_unique_sensors,
                    "best_brain_outputs": summary.best_brain_outputs,
                    "evaluations_completed": summary.evaluations_completed,
                    "champion_file": request
                        .champion_output
                        .map(|path| path.to_string_lossy().to_string()),
                }),
            );
        } else {
            println!(
                "generation {:>4}/{:<4}  best {:>8.4} m  avg {:>8.4} m  segments {}  brain {}",
                summary.generation,
                config.generations,
                summary.best_fitness,
                summary.average_fitness,
                summary.best_segments,
                summary.best_brain_nodes
            );
        }
        Ok(())
    })?;

    let elapsed = started.elapsed();

    if let Some(path) = request.champion_output {
        write_genome(path, &result.champion)?;
    }

    let transport_event = json!({
        "protocol_version": 1,
        "kind": "evolution_complete",
        "champion_fitness": result.champion_fitness,
        "champion_distance": result.champion_distance,
        "champion_segments": result.champion.segments.len(),
        "champion_joints": result.champion.joints.len(),
        "champion_brain_nodes": result.champion.brain.node_count(),
        "champion_brain_sensor_nodes": result.champion.brain.sensor_node_count(),
        "champion_brain_unique_sensors": result.champion.brain.unique_sensor_count(),
        "champion_brain_outputs": result.champion.brain.outputs.len(),
        "generations_completed": result.generations_completed,
        "evaluations_completed": result.evaluations_completed,
        "wall_seconds": elapsed.as_secs_f64(),
        "champion_file": request
            .champion_output
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
                "generations_completed": result.generations_completed,
                "evaluations_completed": result.evaluations_completed,
                "wall_seconds": elapsed.as_secs_f64(),
                "champion": result.champion,
                "history": result.history,
            })
        );
    } else if socket.is_none() {
        println!(
            "complete: champion {:.4} m after {} generations / {} evaluations in {:.3} s",
            result.champion_fitness,
            result.generations_completed,
            result.evaluations_completed,
            elapsed.as_secs_f64()
        );
    }

    Ok(())
}

fn write_genome(path: &PathBuf, genome: &CreatureGenome) -> Result<(), String> {
    let json = serde_json::to_string_pretty(genome)
        .map_err(|err| format!("failed to serialize champion genome: {err}"))?;
    fs::write(path, json)
        .map_err(|err| format!("failed to write champion genome {}: {err}", path.display()))
}

fn run_capabilities(json_output: bool) -> Result<(), String> {
    let backend = RapierCpuBackend;
    let logical_cpu_threads = thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(1);

    let result = json!({
        "protocol_version": 1,
        "kind": "capabilities",
        "app_version": env!("CARGO_PKG_VERSION"),
        "logical_cpu_threads": logical_cpu_threads,
        "backends": [backend.capabilities()],
    });

    if json_output {
        println!("{result}");
    } else {
        println!("Modern 3D Creature Evolution {}", env!("CARGO_PKG_VERSION"));
        println!("logical CPU threads : {logical_cpu_threads}");
        println!("backend             : {}", backend.name());
        println!("state streaming     : yes");
        println!("deterministic       : yes");
        println!("GPU accelerated     : no");
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
