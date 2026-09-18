use std::process::ExitCode;
use std::time::Instant;

use clap::{Parser, Subcommand};
use evolab_core::{BatchRunner, ProbeSpec, RapierCpuBackend, SimulationConfig};

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

            println!("backend              : {}", first.backend);
            println!("worlds evaluated     : {}", reports.len());
            println!("workers requested    : {workers}");
            println!("physics dt           : {:.8} s", config.dt);
            println!("steps/world          : {}", first.steps);
            println!("simulated time/world : {:.3} s", first.simulated_seconds);
            println!("wall time            : {:.3} s", elapsed.as_secs_f64());
            println!("throughput           : {:.1} worlds/s", worlds_per_second);
            println!(
                "final probe position  : [{:.4}, {:.4}, {:.4}]",
                first.final_position[0], first.final_position[1], first.final_position[2]
            );
        }
    }

    Ok(())
}
