use std::net::UdpSocket;
use std::process::ExitCode;
use std::thread;
use std::time::{Duration, Instant};

use clap::{Parser, Subcommand};
use evolab_core::{
    BatchRunner, PhysicsBackend, ProbeSpec, RapierCpuBackend, SimulationConfig, WorldSnapshot,
};
use serde_json::{Value, json};

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
        } => run_stream(&event_host, event_port, seconds, dt, frame_hz, !max_speed),
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
) -> Result<(), String> {
    if !frame_hz.is_finite() || !(1.0..=240.0).contains(&frame_hz) {
        return Err("frame_hz must be between 1 and 240".into());
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
        }),
    );

    let wall_start = Instant::now();
    let mut observer = |snapshot: &WorldSnapshot| -> Result<(), String> {
        if realtime {
            let target = wall_start + Duration::from_secs_f64(snapshot.simulated_seconds as f64);
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
