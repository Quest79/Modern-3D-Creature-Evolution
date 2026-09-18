use rayon::{prelude::*, ThreadPoolBuilder};

use crate::{PhysicsBackend, ProbeSpec, SimulationConfig, SimulationReport};

/// Runs independent simulation worlds in parallel.
///
/// This outer-world parallelism avoids coupling creatures together and maps
/// naturally to future GPU batches and distributed workers.
#[derive(Clone, Copy, Debug)]
pub struct BatchRunner {
    /// 0 means "let Rayon choose".
    pub threads: usize,
}

impl Default for BatchRunner {
    fn default() -> Self {
        Self { threads: 0 }
    }
}

impl BatchRunner {
    pub fn run<B: PhysicsBackend>(
        &self,
        backend: &B,
        config: &SimulationConfig,
        probes: &[ProbeSpec],
    ) -> Result<Vec<SimulationReport>, String> {
        config.validate()?;

        let mut builder = ThreadPoolBuilder::new();
        if self.threads > 0 {
            builder = builder.num_threads(self.threads);
        }

        let pool = builder
            .build()
            .map_err(|err| format!("failed to create simulation worker pool: {err}"))?;

        let results: Vec<Result<SimulationReport, String>> = pool.install(|| {
            probes
                .par_iter()
                .map(|probe| backend.run_probe(config, probe))
                .collect()
        });

        results.into_iter().collect()
    }

    pub fn run_identical<B: PhysicsBackend>(
        &self,
        backend: &B,
        config: &SimulationConfig,
        probe: ProbeSpec,
        count: usize,
    ) -> Result<Vec<SimulationReport>, String> {
        if count == 0 {
            return Err("batch size must be greater than 0".into());
        }

        let probes = vec![probe; count];
        self.run(backend, config, &probes)
    }
}

#[cfg(test)]
mod tests {
    use crate::{BatchRunner, ProbeSpec, RapierCpuBackend, SimulationConfig};

    #[test]
    fn parallel_batch_returns_every_result_in_order() {
        let runner = BatchRunner { threads: 2 };
        let reports = runner
            .run_identical(
                &RapierCpuBackend,
                &SimulationConfig::default(),
                ProbeSpec::default(),
                8,
            )
            .expect("batch should succeed");

        assert_eq!(reports.len(), 8);
        assert!(reports.windows(2).all(|pair| pair[0] == pair[1]));
    }
}
