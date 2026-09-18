use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::{ThreadPoolBuilder, prelude::*};

use crate::{PhysicsBackend, ProbeSpec, SimulationConfig, SimulationReport};

/// Runs independent simulation worlds in parallel.
///
/// This outer-world parallelism avoids coupling creatures together and maps
/// naturally to future GPU batches and distributed workers.
#[derive(Clone, Copy, Debug, Default)]
pub struct BatchRunner {
    /// 0 means "let Rayon choose".
    pub threads: usize,
}

impl BatchRunner {
    pub fn run<B: PhysicsBackend>(
        &self,
        backend: &B,
        config: &SimulationConfig,
        probes: &[ProbeSpec],
    ) -> Result<Vec<SimulationReport>, String> {
        self.run_with_progress(backend, config, probes, |_completed, _total| {})
    }

    pub fn run_with_progress<B, F>(
        &self,
        backend: &B,
        config: &SimulationConfig,
        probes: &[ProbeSpec],
        progress: F,
    ) -> Result<Vec<SimulationReport>, String>
    where
        B: PhysicsBackend,
        F: Fn(usize, usize) + Sync,
    {
        config.validate()?;

        if probes.is_empty() {
            return Err("batch must contain at least one world".into());
        }

        let mut builder = ThreadPoolBuilder::new();
        if self.threads > 0 {
            builder = builder.num_threads(self.threads);
        }

        let pool = builder
            .build()
            .map_err(|err| format!("failed to create simulation worker pool: {err}"))?;

        let completed = AtomicUsize::new(0);
        let total = probes.len();

        let results: Vec<Result<SimulationReport, String>> = pool.install(|| {
            probes
                .par_iter()
                .map(|probe| {
                    let result = backend.run_probe(config, probe);
                    let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
                    progress(done, total);
                    result
                })
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

    pub fn run_identical_with_progress<B, F>(
        &self,
        backend: &B,
        config: &SimulationConfig,
        probe: ProbeSpec,
        count: usize,
        progress: F,
    ) -> Result<Vec<SimulationReport>, String>
    where
        B: PhysicsBackend,
        F: Fn(usize, usize) + Sync,
    {
        if count == 0 {
            return Err("batch size must be greater than 0".into());
        }

        let probes = vec![probe; count];
        self.run_with_progress(backend, config, &probes, progress)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

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

    #[test]
    fn progress_reaches_total() {
        let runner = BatchRunner { threads: 2 };
        let latest = AtomicUsize::new(0);

        runner
            .run_identical_with_progress(
                &RapierCpuBackend,
                &SimulationConfig::default(),
                ProbeSpec::default(),
                8,
                |done, _total| {
                    latest.fetch_max(done, Ordering::Relaxed);
                },
            )
            .unwrap();

        assert_eq!(latest.load(Ordering::Relaxed), 8);
    }
}
