use serde::{Deserialize, Serialize};

use crate::CreatureGenome;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AcceleratorMode {
    #[default]
    Cpu,
    Auto,
    Cuda,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ThroughputMode {
    #[default]
    Deterministic,
    MaxThroughput,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AcceleratorConfig {
    #[serde(default)]
    pub mode: AcceleratorMode,
    #[serde(default)]
    pub gpu_ids: Vec<u32>,
    #[serde(default = "default_gpu_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_max_parts")]
    pub max_parts: usize,
    #[serde(default = "default_max_joints")]
    pub max_joints: usize,
    #[serde(default = "default_true")]
    pub cpu_fallback: bool,
    #[serde(default)]
    pub throughput_mode: ThroughputMode,
}

impl Default for AcceleratorConfig {
    fn default() -> Self {
        Self {
            mode: AcceleratorMode::Cpu,
            gpu_ids: Vec::new(),
            batch_size: default_gpu_batch_size(),
            max_parts: default_max_parts(),
            max_joints: default_max_joints(),
            cpu_fallback: true,
            throughput_mode: ThroughputMode::Deterministic,
        }
    }
}

impl AcceleratorConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.batch_size == 0 {
            return Err("accelerator batch_size must be greater than 0".into());
        }
        if !(1..=1024).contains(&self.max_parts) {
            return Err("accelerator max_parts must be between 1 and 1024".into());
        }
        if !(1..=2048).contains(&self.max_joints) {
            return Err("accelerator max_joints must be between 1 and 2048".into());
        }
        Ok(())
    }

    pub fn selected_gpu_ids(&self, available: &[CudaDeviceInfo]) -> Result<Vec<u32>, String> {
        if self.gpu_ids.is_empty() {
            return Ok(available.iter().map(|device| device.id).collect());
        }

        for requested in &self.gpu_ids {
            if !available.iter().any(|device| device.id == *requested) {
                return Err(format!("CUDA device {requested} is not available"));
            }
        }

        Ok(self.gpu_ids.clone())
    }

    pub fn creature_compatibility(&self, genome: &CreatureGenome) -> GpuCompatibility {
        let mut reasons = Vec::new();
        if genome.segments.len() > self.max_parts {
            reasons.push(format!(
                "{} segments exceeds GPU limit {}",
                genome.segments.len(),
                self.max_parts
            ));
        }
        if genome.joints.len() > self.max_joints {
            reasons.push(format!(
                "{} joints exceeds GPU limit {}",
                genome.joints.len(),
                self.max_joints
            ));
        }

        GpuCompatibility {
            compatible: reasons.is_empty(),
            reasons,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GpuCompatibility {
    pub compatible: bool,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CudaDeviceInfo {
    pub id: u32,
    pub name: String,
    pub total_memory_bytes: u64,
    pub free_memory_bytes: u64,
    pub compute_capability_major: i32,
    pub compute_capability_minor: i32,
    #[serde(default)]
    pub multiprocessor_count: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DeviceWorkAssignment {
    pub device_id: u32,
    pub start_index: usize,
    pub count: usize,
}

pub fn schedule_gpu_work(
    total_items: usize,
    device_ids: &[u32],
) -> Result<Vec<DeviceWorkAssignment>, String> {
    if total_items == 0 {
        return Ok(Vec::new());
    }
    if device_ids.is_empty() {
        return Err("at least one GPU is required to schedule GPU work".into());
    }

    let active_devices = device_ids.len().min(total_items);
    let base = total_items / active_devices;
    let remainder = total_items % active_devices;
    let mut start = 0usize;
    let mut assignments = Vec::with_capacity(active_devices);

    for (slot, device_id) in device_ids.iter().take(active_devices).enumerate() {
        let count = base + usize::from(slot < remainder);
        assignments.push(DeviceWorkAssignment {
            device_id: *device_id,
            start_index: start,
            count,
        });
        start += count;
    }

    Ok(assignments)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct CudaExecutionTelemetry {
    pub host_packing_seconds: f64,
    pub h_to_d_seconds: f64,
    pub kernel_launch_seconds: f64,
    pub kernel_execution_seconds: f64,
    pub synchronization_seconds: f64,
    pub d_to_h_seconds: f64,
    pub result_decode_seconds: f64,
    pub context_creation_seconds: f64,
    pub nvrtc_compile_seconds: f64,
    pub module_load_seconds: f64,
    pub kernel_launch_count: usize,
    pub stream_count: usize,
    pub batch_count: usize,
    pub total_packed_parts: usize,
    pub total_packed_joints: usize,
    pub total_brain_ops: usize,
    pub h_to_d_bytes: u64,
    pub d_to_h_bytes: u64,
    pub allocation_count: u64,
    pub reallocation_count: u64,
    pub device_buffer_capacity_bytes: u64,
    pub unstable_simulations: usize,
    pub block_size: u32,
    pub creature_group_size: u32,
    pub creatures_per_block: u32,
    pub grid_blocks_total: u64,
}

impl CudaExecutionTelemetry {
    pub fn accumulate(&mut self, other: &Self) {
        self.host_packing_seconds += other.host_packing_seconds;
        self.h_to_d_seconds += other.h_to_d_seconds;
        self.kernel_launch_seconds += other.kernel_launch_seconds;
        self.kernel_execution_seconds += other.kernel_execution_seconds;
        self.synchronization_seconds += other.synchronization_seconds;
        self.d_to_h_seconds += other.d_to_h_seconds;
        self.result_decode_seconds += other.result_decode_seconds;
        self.context_creation_seconds += other.context_creation_seconds;
        self.nvrtc_compile_seconds += other.nvrtc_compile_seconds;
        self.module_load_seconds += other.module_load_seconds;
        self.kernel_launch_count += other.kernel_launch_count;
        self.stream_count = self.stream_count.max(other.stream_count);
        self.batch_count += other.batch_count;
        self.total_packed_parts += other.total_packed_parts;
        self.total_packed_joints += other.total_packed_joints;
        self.total_brain_ops += other.total_brain_ops;
        self.h_to_d_bytes += other.h_to_d_bytes;
        self.d_to_h_bytes += other.d_to_h_bytes;
        self.allocation_count += other.allocation_count;
        self.reallocation_count += other.reallocation_count;
        self.device_buffer_capacity_bytes = self
            .device_buffer_capacity_bytes
            .max(other.device_buffer_capacity_bytes);
        self.unstable_simulations += other.unstable_simulations;
        self.block_size = self.block_size.max(other.block_size);
        self.creature_group_size = self.creature_group_size.max(other.creature_group_size);
        self.creatures_per_block = self.creatures_per_block.max(other.creatures_per_block);
        self.grid_blocks_total += other.grid_blocks_total;
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct DevicePerformance {
    pub device_id: u32,
    pub items: usize,
    pub wall_seconds: f64,
    pub items_per_second: f64,
    pub physics_steps_per_second: f64,
    #[serde(default)]
    pub cuda: CudaExecutionTelemetry,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct ExecutionPerformance {
    pub requested_mode: AcceleratorMode,
    pub actual_mode: AcceleratorMode,
    pub cpu_items: usize,
    pub gpu_items: usize,
    pub fallback_items: usize,
    pub wall_seconds: f64,
    pub items_per_second: f64,
    pub physics_steps_per_second: f64,
    pub devices: Vec<DevicePerformance>,
    #[serde(default)]
    pub host_preparation_seconds: f64,
    #[serde(default)]
    pub result_processing_seconds: f64,
    #[serde(default)]
    pub cuda: CudaExecutionTelemetry,
}

const fn default_true() -> bool {
    true
}

const fn default_gpu_batch_size() -> usize {
    4096
}

const fn default_max_parts() -> usize {
    64
}

const fn default_max_joints() -> usize {
    128
}

#[cfg(test)]
mod tests {
    use super::{AcceleratorConfig, CudaDeviceInfo, schedule_gpu_work};
    use crate::CreatureGenome;

    #[test]
    fn scheduler_splits_work_without_losing_items() {
        let assignments = schedule_gpu_work(10, &[0, 1, 2]).unwrap();
        assert_eq!(
            assignments
                .iter()
                .map(|assignment| assignment.count)
                .sum::<usize>(),
            10
        );
        assert_eq!(assignments[0].count, 4);
        assert_eq!(assignments[1].count, 3);
        assert_eq!(assignments[2].count, 3);
    }

    #[test]
    fn default_gpu_limits_accept_seed_creature() {
        let compatibility = AcceleratorConfig::default()
            .creature_compatibility(&CreatureGenome::three_segment_walker());
        assert!(compatibility.compatible);
    }

    #[test]
    fn selected_devices_validate_requested_ids() {
        let config = AcceleratorConfig {
            gpu_ids: vec![1],
            ..AcceleratorConfig::default()
        };
        let devices = vec![CudaDeviceInfo {
            id: 1,
            name: "GPU".into(),
            total_memory_bytes: 1,
            free_memory_bytes: 1,
            compute_capability_major: 0,
            compute_capability_minor: 0,
            multiprocessor_count: 1,
        }];
        assert_eq!(config.selected_gpu_ids(&devices).unwrap(), vec![1]);
    }
}
