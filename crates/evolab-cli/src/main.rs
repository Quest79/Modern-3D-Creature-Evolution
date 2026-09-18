use std::process::ExitCode;
use std::time::Instant;

use clap::{Parser, Subcommand};
use evolab_core::{BatchRunner, ProbeSpec, RapierCpuBackend, SimulationConfig};
use serde_json::json;

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
    /// Validate the Step 1 physics/execution pipeline with independent rigid-body worlds.
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

        /// Emit one machine-readable JSON result for GUI/automation clients.
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
        } => {
            let config = SimulationConfig {
                duration_seconds: seconds,
                dt,
                ..SimulationConfig::default()
            };
            config.validate()?;

            let runner = BatchRunner { threads: workers };
            let backend = RapierCpuBackend;

            let started = Instant::now();
            let reports = runner.run_identical(&backend, &config, ProbeSpec::default(), batch)?;
            let elapsed = started.elapsed();

            let first = &reports[0];
            let worlds_per_second = batch as f64 / elapsed.as_secs_f64();
            let physics_steps = reports.len() as u64 * first.steps as u64;
            let physics_steps_per_second = physics_steps as f64 / elapsed.as_secs_f64();

            if json_output {
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

                println!("{result}");
            } else {
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
        }
    }

    Ok(())
}
