use crate::{
    AcceleratorConfig, CreatureGenome, ExecutionPerformance, FitnessConfig, FitnessResult,
    SimulationConfig, TerrainKind, discover_cuda_devices, schedule_gpu_work,
};

#[cfg(windows)]
use crate::{Expression, SensorKind, legacy_expression};

#[derive(Clone, Debug)]
pub struct CudaCreatureBatchResult {
    pub fitness: Vec<FitnessResult>,
    pub execution: ExecutionPerformance,
}

static CUDA_CREATURE_DEVICE_CACHE: std::sync::OnceLock<Result<Vec<crate::CudaDeviceInfo>, String>> =
    std::sync::OnceLock::new();

fn cached_cuda_devices() -> Result<&'static [crate::CudaDeviceInfo], String> {
    CUDA_CREATURE_DEVICE_CACHE
        .get_or_init(discover_cuda_devices)
        .as_ref()
        .map(Vec::as_slice)
        .map_err(Clone::clone)
}

pub fn run_cuda_creature_batch(
    genomes: &[CreatureGenome],
    simulation: &SimulationConfig,
    fitness: &FitnessConfig,
    accelerator: &AcceleratorConfig,
) -> Result<CudaCreatureBatchResult, String> {
    if genomes.is_empty() {
        return Err("CUDA creature batch must contain at least one genome".into());
    }
    simulation.validate()?;
    fitness.validate()?;
    accelerator.validate()?;
    validate_cuda_creature_world(simulation)?;

    for genome in genomes {
        genome.validate()?;
        let compatibility = accelerator.creature_compatibility(genome);
        if !compatibility.compatible {
            return Err(format!(
                "creature is not compatible with CUDA limits: {}",
                compatibility.reasons.join("; ")
            ));
        }
    }

    // Device topology is stable for the lifetime of the app. Discovering devices
    // creates a temporary CUDA context, so doing it every generation adds large
    // host-side latency that has nothing to do with creature simulation.
    let devices = cached_cuda_devices()?;
    if devices.is_empty() {
        return Err("no CUDA devices were found".into());
    }
    let selected = accelerator.selected_gpu_ids(devices)?;
    if selected.is_empty() {
        return Err("no CUDA devices were selected".into());
    }

    let assignments = schedule_gpu_work(genomes.len(), &selected)?;
    platform::run_batch(
        genomes,
        simulation,
        fitness,
        accelerator,
        devices,
        &assignments,
    )
}

fn validate_cuda_creature_world(config: &SimulationConfig) -> Result<(), String> {
    if config.world.terrain != TerrainKind::Flat {
        return Err("CUDA articulated physics currently requires flat terrain".into());
    }
    if config.world.walls_enabled
        || config.world.blocks_enabled
        || config.world.gaps_enabled
        || config.world.pits_enabled
    {
        return Err("CUDA articulated physics currently requires a world without obstacles".into());
    }
    if config.world.gravity[0].abs() > 1.0e-6 || config.world.gravity[2].abs() > 1.0e-6 {
        return Err("CUDA articulated physics currently requires vertical gravity".into());
    }
    Ok(())
}

#[cfg(windows)]
#[derive(Default)]
struct PackedCreatureBatch {
    world_count: usize,
    part_count: Vec<u32>,
    joint_count: Vec<u32>,
    part_base: Vec<u32>,
    joint_base: Vec<u32>,
    initial_position: Vec<f32>,
    half_extents: Vec<f32>,
    inv_mass: Vec<f32>,
    inv_inertia: Vec<f32>,
    friction: Vec<f32>,
    parent: Vec<u32>,
    child: Vec<u32>,
    axis: Vec<f32>,
    parent_anchor: Vec<f32>,
    child_anchor: Vec<f32>,
    rest_relative: Vec<f32>,
    limit_min: Vec<f32>,
    limit_max: Vec<f32>,
    inertia: Vec<f32>,
    biological_torque: Vec<f32>,
    biological_power: Vec<f32>,
    requested_torque: Vec<f32>,
    brain_start: Vec<u32>,
    brain_count: Vec<u32>,
    op_code: Vec<u32>,
    op_index: Vec<u32>,
    op_a: Vec<f32>,
    op_b: Vec<f32>,
}

#[cfg(windows)]
impl PackedCreatureBatch {
    fn pack(
        genomes: &[CreatureGenome],
        _max_parts: usize,
        _max_joints: usize,
    ) -> Result<Self, String> {
        let world_count = genomes.len();
        let part_slots = genomes
            .iter()
            .map(|genome| genome.segments.len())
            .sum::<usize>();
        let joint_slots = genomes
            .iter()
            .map(|genome| genome.joints.len())
            .sum::<usize>();
        let mut packed = Self {
            world_count,
            part_count: vec![0; world_count],
            joint_count: vec![0; world_count],
            part_base: vec![0; world_count],
            joint_base: vec![0; world_count],
            initial_position: vec![0.0; part_slots * 3],
            half_extents: vec![0.0; part_slots * 3],
            inv_mass: vec![0.0; part_slots],
            inv_inertia: vec![0.0; part_slots],
            friction: vec![0.0; part_slots],
            parent: vec![0; joint_slots],
            child: vec![0; joint_slots],
            axis: vec![0.0; joint_slots * 3],
            parent_anchor: vec![0.0; joint_slots * 3],
            child_anchor: vec![0.0; joint_slots * 3],
            rest_relative: vec![0.0; joint_slots * 3],
            limit_min: vec![0.0; joint_slots],
            limit_max: vec![0.0; joint_slots],
            inertia: vec![0.0; joint_slots],
            biological_torque: vec![0.0; joint_slots],
            biological_power: vec![0.0; joint_slots],
            requested_torque: vec![0.0; joint_slots],
            brain_start: vec![0; joint_slots],
            brain_count: vec![0; joint_slots],
            op_code: Vec::new(),
            op_index: Vec::new(),
            op_a: Vec::new(),
            op_b: Vec::new(),
        };

        let mut next_part = 0usize;
        let mut next_joint = 0usize;
        for (world_index, genome) in genomes.iter().enumerate() {
            packed.part_count[world_index] = genome.segments.len() as u32;
            packed.joint_count[world_index] = genome.joints.len() as u32;
            packed.part_base[world_index] = u32::try_from(next_part)
                .map_err(|_| "CUDA compact part index exceeds u32".to_string())?;
            packed.joint_base[world_index] = u32::try_from(next_joint)
                .map_err(|_| "CUDA compact joint index exceeds u32".to_string())?;

            let mut segment_index = std::collections::HashMap::new();
            for (index, segment) in genome.segments.iter().enumerate() {
                segment_index.insert(segment.id, index as u32);
                let slot = next_part + index;
                for axis in 0..3 {
                    packed.initial_position[slot * 3 + axis] = segment.initial_position[axis];
                    packed.half_extents[slot * 3 + axis] = segment.half_extents[axis];
                }
                let mass = segment.mass_kg().max(1.0e-12);
                packed.inv_mass[slot] = mass.recip();
                let hx = segment.half_extents[0];
                let hy = segment.half_extents[1];
                let hz = segment.half_extents[2];
                let ixx = mass * (hy * hy + hz * hz) / 3.0;
                let iyy = mass * (hx * hx + hz * hz) / 3.0;
                let izz = mass * (hx * hx + hy * hy) / 3.0;
                let average_inertia = ((ixx + iyy + izz) / 3.0).max(1.0e-12);
                packed.inv_inertia[slot] = average_inertia.recip();
                packed.friction[slot] = segment.friction;
            }

            let mut joint_index = std::collections::HashMap::new();
            for (index, joint) in genome.joints.iter().enumerate() {
                joint_index.insert(joint.child_id, index as u32);
            }

            for (index, joint) in genome.joints.iter().enumerate() {
                let slot = next_joint + index;
                let parent = *segment_index
                    .get(&joint.parent_id)
                    .ok_or_else(|| format!("missing parent segment {}", joint.parent_id))?;
                let child = *segment_index
                    .get(&joint.child_id)
                    .ok_or_else(|| format!("missing child segment {}", joint.child_id))?;
                packed.parent[slot] = parent;
                packed.child[slot] = child;
                packed.limit_min[slot] = joint.limits_radians[0];
                packed.limit_max[slot] = joint.limits_radians[1];
                packed.requested_torque[slot] = joint.motor_max_torque;

                let axis_length = joint.axis.iter().map(|v| v * v).sum::<f32>().sqrt();
                for component in 0..3 {
                    packed.axis[slot * 3 + component] = joint.axis[component] / axis_length;
                    packed.parent_anchor[slot * 3 + component] = joint.parent_anchor[component];
                    packed.child_anchor[slot * 3 + component] = joint.child_anchor[component];
                }

                let parent_segment = &genome.segments[parent as usize];
                let child_segment = &genome.segments[child as usize];
                for component in 0..3 {
                    packed.rest_relative[slot * 3 + component] = child_segment.initial_position
                        [component]
                        - parent_segment.initial_position[component];
                }

                let (torque, power) = genome.biological_joint_limits(joint)?;
                packed.biological_torque[slot] = torque;
                packed.biological_power[slot] = power;
                packed.inertia[slot] = genome.joint_effective_inertia(joint)?;

                let expression = genome
                    .brain
                    .output_for_joint(joint.child_id)
                    .cloned()
                    .unwrap_or_else(|| legacy_expression(joint));
                let start = packed.op_code.len();
                flatten_expression(
                    &expression,
                    &joint_index,
                    &segment_index,
                    &mut packed.op_code,
                    &mut packed.op_index,
                    &mut packed.op_a,
                    &mut packed.op_b,
                )?;
                let count = packed.op_code.len() - start;
                packed.brain_start[slot] = u32::try_from(start)
                    .map_err(|_| "CUDA brain bytecode exceeds u32 indexing".to_string())?;
                packed.brain_count[slot] = u32::try_from(count)
                    .map_err(|_| "CUDA brain expression is too large".to_string())?;
            }

            next_part += genome.segments.len();
            next_joint += genome.joints.len();
        }

        Ok(packed)
    }
}

#[cfg(windows)]
fn push_op(
    code: u32,
    index: u32,
    a: f32,
    b: f32,
    op_code: &mut Vec<u32>,
    op_index: &mut Vec<u32>,
    op_a: &mut Vec<f32>,
    op_b: &mut Vec<f32>,
) {
    op_code.push(code);
    op_index.push(index);
    op_a.push(a);
    op_b.push(b);
}

#[cfg(windows)]
fn flatten_expression(
    expression: &Expression,
    joint_index: &std::collections::HashMap<u32, u32>,
    segment_index: &std::collections::HashMap<u32, u32>,
    op_code: &mut Vec<u32>,
    op_index: &mut Vec<u32>,
    op_a: &mut Vec<f32>,
    op_b: &mut Vec<f32>,
) -> Result<(), String> {
    match expression {
        Expression::Constant(value) => push_op(0, 0, *value, 0.0, op_code, op_index, op_a, op_b),
        Expression::Sensor(sensor) => {
            let (code, index) = match sensor {
                SensorKind::Time => (1, 0),
                SensorKind::RootHeight => (2, 0),
                SensorKind::RootVelocityX => (3, 0),
                SensorKind::RootVelocityY => (4, 0),
                SensorKind::RootVelocityZ => (5, 0),
                SensorKind::RootAngularVelocityX => (6, 0),
                SensorKind::RootAngularVelocityY => (7, 0),
                SensorKind::RootAngularVelocityZ => (8, 0),
                SensorKind::RootRotationX => (9, 0),
                SensorKind::RootRotationY => (10, 0),
                SensorKind::RootRotationZ => (11, 0),
                SensorKind::RootRotationW => (12, 0),
                SensorKind::JointAngle(child_id) => (
                    13,
                    *joint_index
                        .get(child_id)
                        .ok_or_else(|| format!("missing joint sensor target {child_id}"))?,
                ),
                SensorKind::JointVelocity(child_id) => (
                    14,
                    *joint_index
                        .get(child_id)
                        .ok_or_else(|| format!("missing joint sensor target {child_id}"))?,
                ),
                SensorKind::SegmentGroundContact(segment_id) => (
                    15,
                    *segment_index
                        .get(segment_id)
                        .ok_or_else(|| format!("missing contact sensor target {segment_id}"))?,
                ),
            };
            push_op(code, index, 0.0, 0.0, op_code, op_index, op_a, op_b);
        }
        Expression::Add(left, right) => {
            flatten_expression(
                left,
                joint_index,
                segment_index,
                op_code,
                op_index,
                op_a,
                op_b,
            )?;
            flatten_expression(
                right,
                joint_index,
                segment_index,
                op_code,
                op_index,
                op_a,
                op_b,
            )?;
            push_op(16, 0, 0.0, 0.0, op_code, op_index, op_a, op_b);
        }
        Expression::Subtract(left, right) => {
            flatten_expression(
                left,
                joint_index,
                segment_index,
                op_code,
                op_index,
                op_a,
                op_b,
            )?;
            flatten_expression(
                right,
                joint_index,
                segment_index,
                op_code,
                op_index,
                op_a,
                op_b,
            )?;
            push_op(17, 0, 0.0, 0.0, op_code, op_index, op_a, op_b);
        }
        Expression::Multiply(left, right) => {
            flatten_expression(
                left,
                joint_index,
                segment_index,
                op_code,
                op_index,
                op_a,
                op_b,
            )?;
            flatten_expression(
                right,
                joint_index,
                segment_index,
                op_code,
                op_index,
                op_a,
                op_b,
            )?;
            push_op(18, 0, 0.0, 0.0, op_code, op_index, op_a, op_b);
        }
        Expression::Negate(value) => {
            flatten_expression(
                value,
                joint_index,
                segment_index,
                op_code,
                op_index,
                op_a,
                op_b,
            )?;
            push_op(19, 0, 0.0, 0.0, op_code, op_index, op_a, op_b);
        }
        Expression::Sin(value) => {
            flatten_expression(
                value,
                joint_index,
                segment_index,
                op_code,
                op_index,
                op_a,
                op_b,
            )?;
            push_op(20, 0, 0.0, 0.0, op_code, op_index, op_a, op_b);
        }
        Expression::Cos(value) => {
            flatten_expression(
                value,
                joint_index,
                segment_index,
                op_code,
                op_index,
                op_a,
                op_b,
            )?;
            push_op(21, 0, 0.0, 0.0, op_code, op_index, op_a, op_b);
        }
        Expression::Clamp { value, min, max } => {
            flatten_expression(
                value,
                joint_index,
                segment_index,
                op_code,
                op_index,
                op_a,
                op_b,
            )?;
            push_op(22, 0, *min, *max, op_code, op_index, op_a, op_b);
        }
    }
    Ok(())
}

#[cfg(not(windows))]
mod platform {
    use crate::{
        AcceleratorConfig, CreatureGenome, CudaDeviceInfo, DeviceWorkAssignment, FitnessConfig,
        SimulationConfig,
    };

    use super::CudaCreatureBatchResult;

    pub fn run_batch(
        _genomes: &[CreatureGenome],
        _simulation: &SimulationConfig,
        _fitness: &FitnessConfig,
        _accelerator: &AcceleratorConfig,
        _devices: &[CudaDeviceInfo],
        _assignments: &[DeviceWorkAssignment],
    ) -> Result<CudaCreatureBatchResult, String> {
        Err("CUDA articulated-creature physics currently supports Windows only".into())
    }
}

#[cfg(windows)]
mod platform {
    use std::{
        collections::HashMap,
        ffi::{CString, OsStr, c_char, c_void},
        mem::{size_of, transmute},
        os::windows::ffi::OsStrExt,
        path::{Path, PathBuf},
        ptr::{null, null_mut},
        sync::{Arc, Mutex, OnceLock},
        thread,
        time::Instant,
    };

    use crate::{
        AcceleratorConfig, AcceleratorMode, CreatureGenome, CudaDeviceInfo, CudaExecutionTelemetry,
        DevicePerformance, DeviceWorkAssignment, ExecutionPerformance, FitnessConfig,
        FitnessMetrics, FitnessResult, SimulationConfig, ThroughputMode,
    };

    use super::{CudaCreatureBatchResult, PackedCreatureBatch};

    type CuResult = i32;
    type CuDevice = i32;
    type CuContext = *mut c_void;
    type CuModule = *mut c_void;
    type CuFunction = *mut c_void;
    type CuStream = *mut c_void;
    type CuDevicePtr = u64;
    type NvrtcResult = i32;
    type NvrtcProgram = *mut c_void;

    const CUDA_SUCCESS: CuResult = 0;
    const NVRTC_SUCCESS: NvrtcResult = 0;

    type KernelCacheKey = (i32, i32, bool);
    static KERNEL_CACHE: OnceLock<Mutex<HashMap<KernelCacheKey, Vec<u8>>>> = OnceLock::new();

    type CuInit = unsafe extern "system" fn(u32) -> CuResult;
    type CuDeviceGet = unsafe extern "system" fn(*mut CuDevice, i32) -> CuResult;
    type CuCtxCreate = unsafe extern "system" fn(*mut CuContext, u32, CuDevice) -> CuResult;
    type CuCtxDestroy = unsafe extern "system" fn(CuContext) -> CuResult;
    type CuCtxSetCurrent = unsafe extern "system" fn(CuContext) -> CuResult;
    type CuStreamCreate = unsafe extern "system" fn(*mut CuStream, u32) -> CuResult;
    type CuStreamDestroy = unsafe extern "system" fn(CuStream) -> CuResult;
    type CuStreamSynchronize = unsafe extern "system" fn(CuStream) -> CuResult;
    type CuMemAlloc = unsafe extern "system" fn(*mut CuDevicePtr, usize) -> CuResult;
    type CuMemFree = unsafe extern "system" fn(CuDevicePtr) -> CuResult;
    type CuMemcpyHtoD = unsafe extern "system" fn(CuDevicePtr, *const c_void, usize) -> CuResult;
    type CuMemcpyDtoH = unsafe extern "system" fn(*mut c_void, CuDevicePtr, usize) -> CuResult;
    type CuModuleLoadData = unsafe extern "system" fn(*mut CuModule, *const c_void) -> CuResult;
    type CuModuleUnload = unsafe extern "system" fn(CuModule) -> CuResult;
    type CuModuleGetFunction =
        unsafe extern "system" fn(*mut CuFunction, CuModule, *const c_char) -> CuResult;
    type CuLaunchKernel = unsafe extern "system" fn(
        CuFunction,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        u32,
        CuStream,
        *mut *mut c_void,
        *mut *mut c_void,
    ) -> CuResult;

    type NvrtcCreateProgram = unsafe extern "system" fn(
        *mut NvrtcProgram,
        *const c_char,
        *const c_char,
        i32,
        *const *const c_char,
        *const *const c_char,
    ) -> NvrtcResult;
    type NvrtcCompileProgram =
        unsafe extern "system" fn(NvrtcProgram, i32, *const *const c_char) -> NvrtcResult;
    type NvrtcGetCubinSize = unsafe extern "system" fn(NvrtcProgram, *mut usize) -> NvrtcResult;
    type NvrtcGetCubin = unsafe extern "system" fn(NvrtcProgram, *mut c_char) -> NvrtcResult;
    type NvrtcGetProgramLogSize =
        unsafe extern "system" fn(NvrtcProgram, *mut usize) -> NvrtcResult;
    type NvrtcGetProgramLog = unsafe extern "system" fn(NvrtcProgram, *mut c_char) -> NvrtcResult;
    type NvrtcDestroyProgram = unsafe extern "system" fn(*mut NvrtcProgram) -> NvrtcResult;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LoadLibraryExW(name: *const u16, file: *mut c_void, flags: u32) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const c_char) -> *mut c_void;
        fn FreeLibrary(module: *mut c_void) -> i32;
        fn GetLastError() -> u32;
    }

    unsafe fn load_symbol(
        library: *mut c_void,
        name: &'static [u8],
    ) -> Result<*mut c_void, String> {
        let pointer = unsafe { GetProcAddress(library, name.as_ptr().cast::<c_char>()) };
        if pointer.is_null() {
            let display = String::from_utf8_lossy(&name[..name.len() - 1]);
            Err(format!("required GPU symbol {display} is unavailable"))
        } else {
            Ok(pointer)
        }
    }

    fn load_library(path: &str) -> Result<*mut c_void, String> {
        const LOAD_WITH_ALTERED_SEARCH_PATH: u32 = 0x0000_0008;

        let wide_path = OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let flags = if Path::new(path).is_absolute() {
            // NVRTC has companion DLLs in the same CUDA bin directory.
            // Make Windows search that directory for dependencies as well.
            LOAD_WITH_ALTERED_SEARCH_PATH
        } else {
            0
        };

        let library = unsafe { LoadLibraryExW(wide_path.as_ptr(), null_mut(), flags) };
        if library.is_null() {
            let error = unsafe { GetLastError() };
            Err(format!(
                "could not load GPU library '{path}' (Windows error {error})"
            ))
        } else {
            Ok(library)
        }
    }

    struct CudaApi {
        library: *mut c_void,
        init: CuInit,
        device_get: CuDeviceGet,
        ctx_create: CuCtxCreate,
        ctx_destroy: CuCtxDestroy,
        ctx_set_current: CuCtxSetCurrent,
        stream_create: CuStreamCreate,
        stream_destroy: CuStreamDestroy,
        stream_synchronize: CuStreamSynchronize,
        mem_alloc: CuMemAlloc,
        mem_free: CuMemFree,
        memcpy_htod: CuMemcpyHtoD,
        memcpy_dtoh: CuMemcpyDtoH,
        module_load_data: CuModuleLoadData,
        module_unload: CuModuleUnload,
        module_get_function: CuModuleGetFunction,
        launch_kernel: CuLaunchKernel,
    }

    unsafe impl Send for CudaApi {}

    impl Drop for CudaApi {
        fn drop(&mut self) {
            if !self.library.is_null() {
                unsafe {
                    FreeLibrary(self.library);
                }
            }
        }
    }

    impl CudaApi {
        fn load() -> Result<Self, String> {
            let library = load_library("nvcuda.dll")?;
            macro_rules! load {
                ($name:literal, $ty:ty) => {{
                    let pointer = unsafe { load_symbol(library, concat!($name, " ").as_bytes())? };
                    unsafe { transmute::<*mut c_void, $ty>(pointer) }
                }};
            }
            let api = Self {
                library,
                init: load!("cuInit", CuInit),
                device_get: load!("cuDeviceGet", CuDeviceGet),
                ctx_create: load!("cuCtxCreate_v2", CuCtxCreate),
                ctx_destroy: load!("cuCtxDestroy_v2", CuCtxDestroy),
                ctx_set_current: load!("cuCtxSetCurrent", CuCtxSetCurrent),
                stream_create: load!("cuStreamCreate", CuStreamCreate),
                stream_destroy: load!("cuStreamDestroy_v2", CuStreamDestroy),
                stream_synchronize: load!("cuStreamSynchronize", CuStreamSynchronize),
                mem_alloc: load!("cuMemAlloc_v2", CuMemAlloc),
                mem_free: load!("cuMemFree_v2", CuMemFree),
                memcpy_htod: load!("cuMemcpyHtoD_v2", CuMemcpyHtoD),
                memcpy_dtoh: load!("cuMemcpyDtoH_v2", CuMemcpyDtoH),
                module_load_data: load!("cuModuleLoadData", CuModuleLoadData),
                module_unload: load!("cuModuleUnload", CuModuleUnload),
                module_get_function: load!("cuModuleGetFunction", CuModuleGetFunction),
                launch_kernel: load!("cuLaunchKernel", CuLaunchKernel),
            };
            check_cuda(unsafe { (api.init)(0) }, "cuInit")?;
            Ok(api)
        }
    }

    struct NvrtcApi {
        library: *mut c_void,
        create_program: NvrtcCreateProgram,
        compile_program: NvrtcCompileProgram,
        get_cubin_size: NvrtcGetCubinSize,
        get_cubin: NvrtcGetCubin,
        get_log_size: NvrtcGetProgramLogSize,
        get_log: NvrtcGetProgramLog,
        destroy_program: NvrtcDestroyProgram,
    }

    impl Drop for NvrtcApi {
        fn drop(&mut self) {
            if !self.library.is_null() {
                unsafe {
                    FreeLibrary(self.library);
                }
            }
        }
    }

    impl NvrtcApi {
        fn load() -> Result<Self, String> {
            let path = find_nvrtc_library().ok_or_else(|| {
                "CUDA Toolkit/NVRTC was not found. Run BOOTSTRAP_AND_RUN.bat again to install CUDA."
                    .to_string()
            })?;
            let display = path.to_string_lossy().into_owned();
            let library = load_library(&display)?;
            macro_rules! load {
                ($name:literal, $ty:ty) => {{
                    let pointer = unsafe { load_symbol(library, concat!($name, " ").as_bytes())? };
                    unsafe { transmute::<*mut c_void, $ty>(pointer) }
                }};
            }
            Ok(Self {
                library,
                create_program: load!("nvrtcCreateProgram", NvrtcCreateProgram),
                compile_program: load!("nvrtcCompileProgram", NvrtcCompileProgram),
                get_cubin_size: load!("nvrtcGetCUBINSize", NvrtcGetCubinSize),
                get_cubin: load!("nvrtcGetCUBIN", NvrtcGetCubin),
                get_log_size: load!("nvrtcGetProgramLogSize", NvrtcGetProgramLogSize),
                get_log: load!("nvrtcGetProgramLog", NvrtcGetProgramLog),
                destroy_program: load!("nvrtcDestroyProgram", NvrtcDestroyProgram),
            })
        }
    }

    fn find_nvrtc_library() -> Option<PathBuf> {
        fn is_wrong_architecture(path: &Path) -> bool {
            let lowercase = path.to_string_lossy().to_ascii_lowercase();
            if cfg!(target_arch = "x86_64") {
                lowercase.contains(r"\arm64\") || lowercase.contains(r"\aarch64\")
            } else if cfg!(target_arch = "aarch64") {
                lowercase.contains(r"\x64\") || lowercase.contains(r"\x86_64\")
            } else {
                false
            }
        }

        fn find_in_bin(bin: &Path) -> Option<PathBuf> {
            if is_wrong_architecture(bin) {
                return None;
            }
            let entries = std::fs::read_dir(bin).ok()?;
            for entry in entries.flatten() {
                let path = entry.path();
                if is_wrong_architecture(&path) {
                    continue;
                }
                let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                    continue;
                };
                let lowercase = name.to_ascii_lowercase();
                if lowercase.starts_with("nvrtc64_") && lowercase.ends_with(".dll") {
                    return Some(path);
                }
            }
            None
        }

        fn find_in_toolkit_root(root: &Path) -> Option<PathBuf> {
            // NVIDIA's normal x64 layout keeps NVRTC directly in bin. Some
            // Toolkit releases also expose an explicit x64 subdirectory.
            for candidate in [root.join("bin"), root.join("bin").join("x64")] {
                if let Some(path) = find_in_bin(&candidate) {
                    return Some(path);
                }
            }
            None
        }

        if let Some(cuda_path) = std::env::var_os("CUDA_PATH") {
            if let Some(path) = find_in_toolkit_root(&PathBuf::from(cuda_path)) {
                return Some(path);
            }
        }

        if let Some(path_value) = std::env::var_os("PATH") {
            for directory in std::env::split_paths(&path_value) {
                if let Some(path) = find_in_bin(&directory) {
                    return Some(path);
                }
            }
        }

        let base = PathBuf::from(r"C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA");
        let mut versions = std::fs::read_dir(base)
            .ok()?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.is_dir())
            .collect::<Vec<_>>();
        versions.sort();
        versions.reverse();

        for version in versions {
            if let Some(path) = find_in_toolkit_root(&version) {
                return Some(path);
            }
        }
        None
    }

    fn check_cuda(result: CuResult, operation: &str) -> Result<(), String> {
        if result == CUDA_SUCCESS {
            Ok(())
        } else {
            Err(format!("{operation} failed with CUDA error code {result}"))
        }
    }

    struct StreamGuard<'a> {
        api: &'a CudaApi,
        stream: CuStream,
    }

    impl Drop for StreamGuard<'_> {
        fn drop(&mut self) {
            if !self.stream.is_null() {
                unsafe {
                    let _ = (self.api.stream_destroy)(self.stream);
                }
            }
        }
    }

    type RuntimeCacheKey = (u32, i32, i32, bool);
    static CUDA_RUNTIME_CACHE: OnceLock<Mutex<HashMap<RuntimeCacheKey, Arc<Mutex<CudaRuntime>>>>> =
        OnceLock::new();

    struct CudaRuntime {
        api: CudaApi,
        context: CuContext,
        module: CuModule,
        function: CuFunction,
        workspace: CudaWorkspace,
    }

    unsafe impl Send for CudaRuntime {}

    impl Drop for CudaRuntime {
        fn drop(&mut self) {
            unsafe {
                let _ = (self.api.ctx_set_current)(self.context);
            }
            self.workspace.release_all(&self.api);
            unsafe {
                if !self.module.is_null() {
                    let _ = (self.api.module_unload)(self.module);
                }
                if !self.context.is_null() {
                    let _ = (self.api.ctx_destroy)(self.context);
                }
            }
        }
    }

    fn get_or_create_runtime(
        device_info: &CudaDeviceInfo,
        throughput_mode: ThroughputMode,
    ) -> Result<(Arc<Mutex<CudaRuntime>>, CudaExecutionTelemetry), String> {
        let key = (
            device_info.id,
            device_info.compute_capability_major,
            device_info.compute_capability_minor,
            throughput_mode == ThroughputMode::MaxThroughput,
        );
        let cache = CUDA_RUNTIME_CACHE.get_or_init(|| Mutex::new(HashMap::new()));

        if let Some(runtime) = cache
            .lock()
            .map_err(|_| "CUDA runtime cache mutex was poisoned".to_string())?
            .get(&key)
            .cloned()
        {
            return Ok((runtime, CudaExecutionTelemetry::default()));
        }

        let mut initialization = CudaExecutionTelemetry::default();

        let api = CudaApi::load()?;
        let mut device = 0;
        check_cuda(
            unsafe { (api.device_get)(&mut device, device_info.id as i32) },
            "cuDeviceGet",
        )?;

        let mut context = null_mut();
        let context_started = Instant::now();
        check_cuda(
            unsafe { (api.ctx_create)(&mut context, 0, device) },
            "cuCtxCreate",
        )?;
        initialization.context_creation_seconds = context_started.elapsed().as_secs_f64();

        let compile_started = Instant::now();
        let image = compile_kernel(
            device_info.compute_capability_major,
            device_info.compute_capability_minor,
            throughput_mode,
        )?;
        initialization.nvrtc_compile_seconds = compile_started.elapsed().as_secs_f64();

        let mut module = null_mut();
        let module_started = Instant::now();
        if let Err(error) = check_cuda(
            unsafe { (api.module_load_data)(&mut module, image.as_ptr().cast::<c_void>()) },
            "cuModuleLoadData",
        ) {
            unsafe {
                let _ = (api.ctx_destroy)(context);
            }
            return Err(error);
        }

        let kernel_name = CString::new("simulate_creatures").expect("static kernel name");
        let mut function = null_mut();
        if let Err(error) = check_cuda(
            unsafe { (api.module_get_function)(&mut function, module, kernel_name.as_ptr()) },
            "cuModuleGetFunction",
        ) {
            unsafe {
                let _ = (api.module_unload)(module);
                let _ = (api.ctx_destroy)(context);
            }
            return Err(error);
        }

        initialization.module_load_seconds = module_started.elapsed().as_secs_f64();

        let runtime = Arc::new(Mutex::new(CudaRuntime {
            api,
            context,
            module,
            function,
            workspace: CudaWorkspace::default(),
        }));

        let mut guard = cache
            .lock()
            .map_err(|_| "CUDA runtime cache mutex was poisoned".to_string())?;
        let runtime = guard.entry(key).or_insert_with(|| runtime.clone()).clone();
        Ok((runtime, initialization))
    }

    #[derive(Default)]
    struct RawDeviceBuffer {
        pointer: CuDevicePtr,
        capacity_bytes: usize,
    }

    impl RawDeviceBuffer {
        fn ensure_capacity(&mut self, api: &CudaApi, required_bytes: usize) -> Result<(), String> {
            let required_bytes = required_bytes.max(1);
            if self.capacity_bytes >= required_bytes && self.pointer != 0 {
                return Ok(());
            }

            if self.pointer != 0 {
                check_cuda(unsafe { (api.mem_free)(self.pointer) }, "cuMemFree")?;
                self.pointer = 0;
                self.capacity_bytes = 0;
            }

            // Grow geometrically so normal generation-to-generation morphology
            // changes do not force another device allocation.
            let capacity_bytes = required_bytes.next_power_of_two();
            check_cuda(
                unsafe { (api.mem_alloc)(&mut self.pointer, capacity_bytes) },
                "cuMemAlloc",
            )?;
            self.capacity_bytes = capacity_bytes;
            Ok(())
        }
    }

    #[derive(Default)]
    struct CudaWorkspace {
        buffers: HashMap<&'static str, RawDeviceBuffer>,
        allocation_count: u64,
        reallocation_count: u64,
        h_to_d_bytes: u64,
        d_to_h_bytes: u64,
    }

    impl CudaWorkspace {
        fn ensure(
            &mut self,
            api: &CudaApi,
            name: &'static str,
            required_bytes: usize,
        ) -> Result<CuDevicePtr, String> {
            let buffer = self.buffers.entry(name).or_default();
            let had_allocation = buffer.pointer != 0;
            let previous_capacity = buffer.capacity_bytes;
            buffer.ensure_capacity(api, required_bytes)?;
            if buffer.capacity_bytes != previous_capacity {
                self.allocation_count += 1;
                if had_allocation {
                    self.reallocation_count += 1;
                }
            }
            Ok(buffer.pointer)
        }

        fn upload<T>(
            &mut self,
            api: &CudaApi,
            name: &'static str,
            values: &[T],
        ) -> Result<CuDevicePtr, String> {
            let bytes = values.len().max(1) * size_of::<T>();
            let pointer = self.ensure(api, name, bytes)?;
            if !values.is_empty() {
                let transfer_bytes = values.len() * size_of::<T>();
                check_cuda(
                    unsafe {
                        (api.memcpy_htod)(pointer, values.as_ptr().cast::<c_void>(), transfer_bytes)
                    },
                    "cuMemcpyHtoD",
                )?;
                self.h_to_d_bytes += transfer_bytes as u64;
            }
            Ok(pointer)
        }

        fn download<T>(
            &mut self,
            api: &CudaApi,
            name: &'static str,
            values: &mut [T],
        ) -> Result<(), String> {
            let (pointer, capacity_bytes) = {
                let buffer = self
                    .buffers
                    .get(name)
                    .ok_or_else(|| format!("CUDA workspace buffer '{name}' is missing"))?;
                (buffer.pointer, buffer.capacity_bytes)
            };
            let bytes = values.len() * size_of::<T>();
            if bytes > capacity_bytes {
                return Err(format!(
                    "CUDA workspace buffer '{name}' is too small: {} < {} bytes",
                    capacity_bytes, bytes
                ));
            }
            if !values.is_empty() {
                check_cuda(
                    unsafe {
                        (api.memcpy_dtoh)(values.as_mut_ptr().cast::<c_void>(), pointer, bytes)
                    },
                    "cuMemcpyDtoH",
                )?;
                self.d_to_h_bytes += bytes as u64;
            }
            Ok(())
        }

        fn total_capacity_bytes(&self) -> u64 {
            self.buffers
                .values()
                .map(|buffer| buffer.capacity_bytes as u64)
                .sum()
        }

        fn release_all(&mut self, api: &CudaApi) {
            for buffer in self.buffers.values_mut() {
                if buffer.pointer != 0 {
                    unsafe {
                        let _ = (api.mem_free)(buffer.pointer);
                    }
                    buffer.pointer = 0;
                    buffer.capacity_bytes = 0;
                }
            }
            self.buffers.clear();
        }
    }

    struct NvrtcProgramGuard<'a> {
        api: &'a NvrtcApi,
        program: NvrtcProgram,
    }

    impl Drop for NvrtcProgramGuard<'_> {
        fn drop(&mut self) {
            if !self.program.is_null() {
                unsafe {
                    let _ = (self.api.destroy_program)(&mut self.program);
                }
            }
        }
    }

    fn compile_kernel(
        major: i32,
        minor: i32,
        throughput_mode: ThroughputMode,
    ) -> Result<Vec<u8>, String> {
        let cache_key = (
            major,
            minor,
            throughput_mode == ThroughputMode::MaxThroughput,
        );
        let cache = KERNEL_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        if let Some(image) = cache
            .lock()
            .map_err(|_| "CUDA kernel cache mutex was poisoned".to_string())?
            .get(&cache_key)
            .cloned()
        {
            return Ok(image);
        }

        let api = NvrtcApi::load()?;
        let source = CString::new(CUDA_CREATURE_SOURCE).expect("CUDA source has no NUL");
        let name = CString::new("evolab_creature.cu").expect("static name");
        let mut program = null_mut();
        let create = unsafe {
            (api.create_program)(
                &mut program,
                source.as_ptr(),
                name.as_ptr(),
                0,
                null(),
                null(),
            )
        };
        if create != NVRTC_SUCCESS {
            return Err(format!("nvrtcCreateProgram failed with code {create}"));
        }
        let guard = NvrtcProgramGuard { api: &api, program };

        // Compile directly to native SASS/cubin for the selected GPU instead
        // of emitting PTX. This avoids CUDA error 222 when a newer Toolkit
        // emits a PTX ISA revision that the installed display driver cannot JIT.
        let arch = CString::new(format!("--gpu-architecture=sm_{major}{minor}"))
            .map_err(|_| "invalid CUDA architecture".to_string())?;
        let std = CString::new("--std=c++14").expect("static option");
        let fast_math = CString::new("--use_fast_math").expect("static option");
        let mut option_storage = vec![arch, std];
        if throughput_mode == ThroughputMode::MaxThroughput {
            option_storage.push(fast_math);
        }
        let option_ptrs = option_storage
            .iter()
            .map(|option| option.as_ptr())
            .collect::<Vec<_>>();

        let compile = unsafe {
            (api.compile_program)(program, option_ptrs.len() as i32, option_ptrs.as_ptr())
        };
        if compile != NVRTC_SUCCESS {
            let mut log_size = 0usize;
            unsafe {
                let _ = (api.get_log_size)(program, &mut log_size);
            }
            let mut log = vec![0_u8; log_size.max(1)];
            unsafe {
                let _ = (api.get_log)(program, log.as_mut_ptr().cast::<c_char>());
            }
            let message = String::from_utf8_lossy(&log)
                .trim_end_matches(' ')
                .trim()
                .to_string();
            return Err(format!(
                "NVRTC failed to compile CUDA creature solver: {message}"
            ));
        }

        let mut cubin_size = 0usize;
        let cubin_size_result = unsafe { (api.get_cubin_size)(program, &mut cubin_size) };
        if cubin_size_result != NVRTC_SUCCESS {
            return Err(format!(
                "nvrtcGetCUBINSize failed with code {cubin_size_result}"
            ));
        }
        let mut cubin = vec![0_u8; cubin_size.max(1)];
        let cubin_result = unsafe { (api.get_cubin)(program, cubin.as_mut_ptr().cast::<c_char>()) };
        if cubin_result != NVRTC_SUCCESS {
            return Err(format!("nvrtcGetCUBIN failed with code {cubin_result}"));
        }
        drop(guard);

        cache
            .lock()
            .map_err(|_| "CUDA kernel cache mutex was poisoned".to_string())?
            .insert(cache_key, cubin.clone());
        Ok(cubin)
    }

    pub fn run_batch(
        genomes: &[CreatureGenome],
        simulation: &SimulationConfig,
        fitness: &FitnessConfig,
        accelerator: &AcceleratorConfig,
        devices: &[CudaDeviceInfo],
        assignments: &[DeviceWorkAssignment],
    ) -> Result<CudaCreatureBatchResult, String> {
        let started = Instant::now();
        let mut handles = Vec::with_capacity(assignments.len());

        for assignment in assignments {
            let assignment = assignment.clone();
            let genomes =
                genomes[assignment.start_index..assignment.start_index + assignment.count].to_vec();
            let simulation = simulation.clone();
            let fitness = *fitness;
            let accelerator = accelerator.clone();
            let device = devices
                .iter()
                .find(|device| device.id == assignment.device_id)
                .cloned()
                .ok_or_else(|| format!("CUDA device {} disappeared", assignment.device_id))?;
            handles.push(thread::spawn(move || {
                run_device_assignment(
                    &genomes,
                    &simulation,
                    &fitness,
                    &accelerator,
                    &device,
                    assignment.start_index,
                )
            }));
        }

        let mut results = vec![
            FitnessResult {
                score: -1.0e30,
                metrics: FitnessMetrics::default(),
            };
            genomes.len()
        ];
        let mut device_stats = Vec::new();

        for handle in handles {
            let result = handle
                .join()
                .map_err(|_| "CUDA creature worker thread panicked".to_string())??;
            for (index, fitness) in result.results {
                results[index] = fitness;
            }
            device_stats.push(result.performance);
        }

        let wall_seconds = started.elapsed().as_secs_f64().max(f64::EPSILON);
        let physics_steps = genomes.len() as u64 * simulation.step_count() as u64;
        Ok(CudaCreatureBatchResult {
            fitness: results,
            execution: {
                let mut cuda = CudaExecutionTelemetry::default();
                for device in &device_stats {
                    cuda.accumulate(&device.cuda);
                }
                ExecutionPerformance {
                    requested_mode: accelerator.mode,
                    actual_mode: AcceleratorMode::Cuda,
                    cpu_items: 0,
                    gpu_items: genomes.len(),
                    fallback_items: 0,
                    wall_seconds,
                    items_per_second: genomes.len() as f64 / wall_seconds,
                    physics_steps_per_second: physics_steps as f64 / wall_seconds,
                    devices: device_stats,
                    host_preparation_seconds: 0.0,
                    result_processing_seconds: 0.0,
                    cuda,
                }
            },
        })
    }

    struct DeviceAssignmentResult {
        results: Vec<(usize, FitnessResult)>,
        performance: DevicePerformance,
    }

    fn run_device_assignment(
        genomes: &[CreatureGenome],
        simulation: &SimulationConfig,
        fitness: &FitnessConfig,
        accelerator: &AcceleratorConfig,
        device_info: &CudaDeviceInfo,
        global_start: usize,
    ) -> Result<DeviceAssignmentResult, String> {
        let (runtime, mut cuda_telemetry) =
            get_or_create_runtime(device_info, accelerator.throughput_mode)?;
        let mut runtime = runtime
            .lock()
            .map_err(|_| "CUDA device runtime mutex was poisoned".to_string())?;
        check_cuda(
            unsafe { (runtime.api.ctx_set_current)(runtime.context) },
            "cuCtxSetCurrent",
        )?;
        let started = Instant::now();
        let mut all_results = Vec::with_capacity(genomes.len());

        // A generation is one fixed evolutionary workload. Keep every candidate
        // assigned to this GPU in the same CUDA launch instead of serializing it
        // into smaller chunks based on a UI batch-size knob.
        let batch_size = genomes.len().max(1);
        let mut offset = 0usize;
        while offset < genomes.len() {
            let end = (offset + batch_size).min(genomes.len());
            let chunk = &genomes[offset..end];
            let chunk_result = run_chunk(&mut runtime, chunk, simulation, fitness, accelerator)?;
            cuda_telemetry.accumulate(&chunk_result.telemetry);
            all_results.extend(
                chunk_result
                    .fitness
                    .into_iter()
                    .enumerate()
                    .map(|(index, result)| (global_start + offset + index, result)),
            );
            offset = end;
        }

        let wall_seconds = started.elapsed().as_secs_f64().max(f64::EPSILON);
        let physics_steps = genomes.len() as u64 * simulation.step_count() as u64;
        Ok(DeviceAssignmentResult {
            results: all_results,
            performance: DevicePerformance {
                device_id: device_info.id,
                items: genomes.len(),
                wall_seconds,
                items_per_second: genomes.len() as f64 / wall_seconds,
                physics_steps_per_second: physics_steps as f64 / wall_seconds,
                cuda: cuda_telemetry,
            },
        })
    }

    struct ChunkResult {
        fitness: Vec<FitnessResult>,
        telemetry: CudaExecutionTelemetry,
    }

    fn run_chunk(
        runtime: &mut CudaRuntime,
        genomes: &[CreatureGenome],
        simulation: &SimulationConfig,
        fitness: &FitnessConfig,
        accelerator: &AcceleratorConfig,
    ) -> Result<ChunkResult, String> {
        let mut telemetry = CudaExecutionTelemetry::default();
        telemetry.batch_count = 1;

        let packing_started = Instant::now();
        let packed =
            PackedCreatureBatch::pack(genomes, accelerator.max_parts, accelerator.max_joints)?;
        telemetry.host_packing_seconds = packing_started.elapsed().as_secs_f64();
        let world_count = packed.world_count;
        let part_slots = packed.inv_mass.len();
        let joint_slots = packed.parent.len();
        telemetry.total_packed_parts = part_slots;
        telemetry.total_packed_joints = joint_slots;
        telemetry.total_brain_ops = packed.op_code.len();

        let api = &runtime.api;
        let function = runtime.function;
        let workspace = &mut runtime.workspace;
        let allocation_before = workspace.allocation_count;
        let reallocation_before = workspace.reallocation_count;
        let h_to_d_before = workspace.h_to_d_bytes;
        let d_to_h_before = workspace.d_to_h_bytes;

        let upload_started = Instant::now();
        let mut p_part_count = workspace.upload(api, "part_count", &packed.part_count)?;
        let mut p_joint_count = workspace.upload(api, "joint_count", &packed.joint_count)?;
        let mut p_part_base = workspace.upload(api, "part_base", &packed.part_base)?;
        let mut p_joint_base = workspace.upload(api, "joint_base", &packed.joint_base)?;
        let mut p_initial_position =
            workspace.upload(api, "initial_position", &packed.initial_position)?;
        let mut p_half_extents = workspace.upload(api, "half_extents", &packed.half_extents)?;
        let mut p_inv_mass = workspace.upload(api, "inv_mass", &packed.inv_mass)?;
        let mut p_inv_inertia = workspace.upload(api, "inv_inertia", &packed.inv_inertia)?;
        let mut p_friction = workspace.upload(api, "friction", &packed.friction)?;
        let mut p_parent = workspace.upload(api, "parent", &packed.parent)?;
        let mut p_child = workspace.upload(api, "child", &packed.child)?;
        let mut p_axis = workspace.upload(api, "axis", &packed.axis)?;
        let mut p_parent_anchor = workspace.upload(api, "parent_anchor", &packed.parent_anchor)?;
        let mut p_child_anchor = workspace.upload(api, "child_anchor", &packed.child_anchor)?;
        let mut p_rest_relative = workspace.upload(api, "rest_relative", &packed.rest_relative)?;
        let mut p_limit_min = workspace.upload(api, "limit_min", &packed.limit_min)?;
        let mut p_limit_max = workspace.upload(api, "limit_max", &packed.limit_max)?;
        let mut p_inertia = workspace.upload(api, "inertia", &packed.inertia)?;
        let mut p_biological_torque =
            workspace.upload(api, "biological_torque", &packed.biological_torque)?;
        let mut p_biological_power =
            workspace.upload(api, "biological_power", &packed.biological_power)?;
        let mut p_requested_torque =
            workspace.upload(api, "requested_torque", &packed.requested_torque)?;
        let mut p_brain_start = workspace.upload(api, "brain_start", &packed.brain_start)?;
        let mut p_brain_count = workspace.upload(api, "brain_count", &packed.brain_count)?;
        let mut p_op_code = workspace.upload(api, "op_code", &packed.op_code)?;
        let mut p_op_index = workspace.upload(api, "op_index", &packed.op_index)?;
        let mut p_op_a = workspace.upload(api, "op_a", &packed.op_a)?;
        let mut p_op_b = workspace.upload(api, "op_b", &packed.op_b)?;
        telemetry.h_to_d_seconds = upload_started.elapsed().as_secs_f64();

        let mut p_state_position =
            workspace.ensure(api, "state_position", part_slots * 3 * size_of::<f32>())?;
        let mut p_state_velocity =
            workspace.ensure(api, "state_velocity", part_slots * 3 * size_of::<f32>())?;
        let mut p_state_rotation =
            workspace.ensure(api, "state_rotation", part_slots * 4 * size_of::<f32>())?;
        let mut p_state_angular_velocity =
            workspace.ensure(api, "state_angular_velocity", part_slots * 3 * size_of::<f32>())?;
        let mut p_state_contact =
            workspace.ensure(api, "state_contact", part_slots * size_of::<f32>())?;
        let mut p_state_angle =
            workspace.ensure(api, "state_angle", joint_slots * size_of::<f32>())?;
        let mut p_state_angvel =
            workspace.ensure(api, "state_angvel", joint_slots * size_of::<f32>())?;
        let mut p_state_target =
            workspace.ensure(api, "state_target", joint_slots * size_of::<f32>())?;
        let mut p_joint_force =
            workspace.ensure(api, "joint_force", joint_slots * 3 * size_of::<f32>())?;
        let mut p_joint_torque =
            workspace.ensure(api, "joint_torque", joint_slots * 3 * size_of::<f32>())?;

        let mut p_out_score = workspace.ensure(api, "out_score", world_count * size_of::<f32>())?;
        let mut p_out_distance =
            workspace.ensure(api, "out_distance", world_count * size_of::<f32>())?;
        let mut p_out_speed = workspace.ensure(api, "out_speed", world_count * size_of::<f32>())?;
        let mut p_out_upright =
            workspace.ensure(api, "out_upright", world_count * size_of::<f32>())?;
        let mut p_out_stability =
            workspace.ensure(api, "out_stability", world_count * size_of::<f32>())?;
        let mut p_out_energy =
            workspace.ensure(api, "out_energy", world_count * size_of::<f32>())?;
        let mut p_out_unstable =
            workspace.ensure(api, "out_unstable", world_count * size_of::<u32>())?;

        let mut dt = simulation.dt;
        let mut steps = simulation.step_count() as u32;
        let mut gravity_y = simulation.world.gravity[1];
        let mut ground_y = simulation.world.surface_height_at(0.0, 0.0).unwrap_or(0.0);
        let mut activation = simulation.motor_strength_multiplier;
        let mut weight_distance = fitness.weights.distance;
        let mut weight_speed = fitness.weights.average_speed;
        let mut weight_upright = fitness.weights.upright;
        let mut weight_stability = fitness.weights.stability;
        let mut weight_energy = fitness.weights.energy;

        // One full warp owns one creature. With one warp/block, a 50-creature
        // generation launches 50 independent blocks so the GPU scheduler can
        // place the whole generation across the SMs concurrently.
        const CUDA_CONCURRENT_LANES: usize = 1;
        const CUDA_GROUP_SIZE: u32 = 32;
        const CUDA_BLOCK_SIZE: u32 = 32;
        const CUDA_CREATURES_PER_BLOCK: u32 = 1;
        telemetry.block_size = CUDA_BLOCK_SIZE;
        telemetry.creature_group_size = CUDA_GROUP_SIZE;
        telemetry.creatures_per_block = CUDA_CREATURES_PER_BLOCK;

        let lane_count = world_count.min(CUDA_CONCURRENT_LANES).max(1);
        telemetry.stream_count = lane_count;
        let mut streams = Vec::with_capacity(lane_count);
        for _ in 0..lane_count {
            let mut stream = null_mut();
            check_cuda(
                unsafe { (api.stream_create)(&mut stream, 0) },
                "cuStreamCreate",
            )?;
            streams.push(StreamGuard { api, stream });
        }

        let kernel_window_started = Instant::now();
        let launch_started = Instant::now();
        for (lane, stream) in streams.iter().enumerate() {
            let start = world_count * lane / lane_count;
            let end = world_count * (lane + 1) / lane_count;
            let launch_count = end - start;
            if launch_count == 0 {
                continue;
            }

            let mut world_start_arg = start as u32;
            let mut launch_count_arg = launch_count as u32;
            let mut params = [
                (&mut p_part_count as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_joint_count as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_part_base as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_joint_base as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_initial_position as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_half_extents as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_inv_mass as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_inv_inertia as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_friction as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_parent as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_child as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_axis as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_parent_anchor as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_child_anchor as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_rest_relative as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_limit_min as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_limit_max as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_inertia as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_biological_torque as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_biological_power as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_requested_torque as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_brain_start as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_brain_count as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_op_code as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_op_index as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_op_a as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_op_b as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_state_position as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_state_velocity as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_state_rotation as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_state_angular_velocity as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_state_contact as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_state_angle as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_state_angvel as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_state_target as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_joint_force as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_joint_torque as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_out_score as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_out_distance as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_out_speed as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_out_upright as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_out_stability as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_out_energy as *mut CuDevicePtr).cast::<c_void>(),
                (&mut p_out_unstable as *mut CuDevicePtr).cast::<c_void>(),
                (&mut world_start_arg as *mut u32).cast::<c_void>(),
                (&mut launch_count_arg as *mut u32).cast::<c_void>(),
                (&mut dt as *mut f32).cast::<c_void>(),
                (&mut steps as *mut u32).cast::<c_void>(),
                (&mut gravity_y as *mut f32).cast::<c_void>(),
                (&mut ground_y as *mut f32).cast::<c_void>(),
                (&mut activation as *mut f32).cast::<c_void>(),
                (&mut weight_distance as *mut f32).cast::<c_void>(),
                (&mut weight_speed as *mut f32).cast::<c_void>(),
                (&mut weight_upright as *mut f32).cast::<c_void>(),
                (&mut weight_stability as *mut f32).cast::<c_void>(),
                (&mut weight_energy as *mut f32).cast::<c_void>(),
            ];

            let grid_size = (launch_count as u32).div_ceil(CUDA_CREATURES_PER_BLOCK);
            telemetry.kernel_launch_count += 1;
            telemetry.grid_blocks_total += grid_size as u64;
            check_cuda(
                unsafe {
                    (api.launch_kernel)(
                        function,
                        grid_size,
                        1,
                        1,
                        CUDA_BLOCK_SIZE,
                        1,
                        1,
                        0,
                        stream.stream,
                        params.as_mut_ptr(),
                        null_mut(),
                    )
                },
                "cuLaunchKernel(simulate_creatures)",
            )?;
        }
        telemetry.kernel_launch_seconds = launch_started.elapsed().as_secs_f64();

        let synchronization_started = Instant::now();
        for stream in &streams {
            check_cuda(
                unsafe { (api.stream_synchronize)(stream.stream) },
                "cuStreamSynchronize",
            )?;
        }
        telemetry.synchronization_seconds = synchronization_started.elapsed().as_secs_f64();
        telemetry.kernel_execution_seconds = kernel_window_started.elapsed().as_secs_f64();

        let mut score = vec![0.0; world_count];
        let mut distance = vec![0.0; world_count];
        let mut speed = vec![0.0; world_count];
        let mut upright = vec![0.0; world_count];
        let mut stability = vec![0.0; world_count];
        let mut energy = vec![0.0; world_count];
        let mut unstable = vec![0_u32; world_count];
        let download_started = Instant::now();
        workspace.download(api, "out_score", &mut score)?;
        workspace.download(api, "out_distance", &mut distance)?;
        workspace.download(api, "out_speed", &mut speed)?;
        workspace.download(api, "out_upright", &mut upright)?;
        workspace.download(api, "out_stability", &mut stability)?;
        workspace.download(api, "out_energy", &mut energy)?;
        workspace.download(api, "out_unstable", &mut unstable)?;
        telemetry.d_to_h_seconds = download_started.elapsed().as_secs_f64();

        let decode_started = Instant::now();
        telemetry.unstable_simulations = unstable.iter().filter(|value| **value != 0).count();
        let decoded = (0..world_count)
            .map(|index| {
                if unstable[index] != 0 {
                    FitnessResult {
                        score: -1.0e30,
                        metrics: FitnessMetrics::default(),
                    }
                } else {
                    FitnessResult {
                        score: score[index],
                        metrics: FitnessMetrics {
                            distance: distance[index],
                            average_speed: speed[index],
                            upright: upright[index],
                            stability: stability[index],
                            energy: energy[index],
                        },
                    }
                }
            })
            .collect();
        telemetry.result_decode_seconds = decode_started.elapsed().as_secs_f64();
        telemetry.h_to_d_bytes = workspace.h_to_d_bytes.saturating_sub(h_to_d_before);
        telemetry.d_to_h_bytes = workspace.d_to_h_bytes.saturating_sub(d_to_h_before);
        telemetry.allocation_count = workspace.allocation_count.saturating_sub(allocation_before);
        telemetry.reallocation_count = workspace
            .reallocation_count
            .saturating_sub(reallocation_before);
        telemetry.device_buffer_capacity_bytes = workspace.total_capacity_bytes();

        Ok(ChunkResult {
            fitness: decoded,
            telemetry,
        })
    }

    const CUDA_CREATURE_SOURCE: &str = r#"
extern "C" __device__ float clampf(float x, float lo, float hi) {
    return fminf(fmaxf(x, lo), hi);
}

extern "C" __device__ float sane(float x) {
    if (!(x == x) || fabsf(x) > 1000.0f) return 0.0f;
    return clampf(x, -1000.0f, 1000.0f);
}

extern "C" __device__ void rotate_vec(
    float qx, float qy, float qz, float qw,
    float vx, float vy, float vz,
    float* ox, float* oy, float* oz
) {
    float tx = 2.0f * (qy * vz - qz * vy);
    float ty = 2.0f * (qz * vx - qx * vz);
    float tz = 2.0f * (qx * vy - qy * vx);
    *ox = vx + qw * tx + (qy * tz - qz * ty);
    *oy = vy + qw * ty + (qz * tx - qx * tz);
    *oz = vz + qw * tz + (qx * ty - qy * tx);
}

extern "C" __device__ void integrate_quat(
    float* rotation,
    unsigned slot,
    float wx,
    float wy,
    float wz,
    float dt
) {
    unsigned base = slot * 4;
    float qx = rotation[base + 0];
    float qy = rotation[base + 1];
    float qz = rotation[base + 2];
    float qw = rotation[base + 3];

    float dx = 0.5f * ( wx * qw + wy * qz - wz * qy);
    float dy = 0.5f * (-wx * qz + wy * qw + wz * qx);
    float dz = 0.5f * ( wx * qy - wy * qx + wz * qw);
    float dw = 0.5f * (-wx * qx - wy * qy - wz * qz);

    qx += dx * dt;
    qy += dy * dt;
    qz += dz * dt;
    qw += dw * dt;

    float n2 = qx * qx + qy * qy + qz * qz + qw * qw;
    if (!(n2 == n2) || n2 < 1.0e-12f) {
        qx = 0.0f; qy = 0.0f; qz = 0.0f; qw = 1.0f;
    } else {
        float inv_n = rsqrtf(n2);
        qx *= inv_n; qy *= inv_n; qz *= inv_n; qw *= inv_n;
    }

    rotation[base + 0] = qx;
    rotation[base + 1] = qy;
    rotation[base + 2] = qz;
    rotation[base + 3] = qw;
}

extern "C" __device__ float projected_half_height(
    const float* rotation,
    const float* half_extents,
    unsigned slot
) {
    unsigned qb = slot * 4;
    float qx = rotation[qb + 0];
    float qy = rotation[qb + 1];
    float qz = rotation[qb + 2];
    float qw = rotation[qb + 3];

    float exx, exy, exz;
    float eyx, eyy, eyz;
    float ezx, ezy, ezz;
    rotate_vec(qx, qy, qz, qw, 1.0f, 0.0f, 0.0f, &exx, &exy, &exz);
    rotate_vec(qx, qy, qz, qw, 0.0f, 1.0f, 0.0f, &eyx, &eyy, &eyz);
    rotate_vec(qx, qy, qz, qw, 0.0f, 0.0f, 1.0f, &ezx, &ezy, &ezz);

    return fabsf(exy) * half_extents[slot * 3 + 0]
        + fabsf(eyy) * half_extents[slot * 3 + 1]
        + fabsf(ezy) * half_extents[slot * 3 + 2];
}

extern "C" __device__ float eval_brain(
    unsigned start,
    unsigned count,
    const unsigned* op_code,
    const unsigned* op_index,
    const float* op_a,
    const float* op_b,
    float time_seconds,
    const float* position,
    const float* velocity,
    const float* rotation,
    const float* angular_velocity,
    const float* joint_angle,
    const float* joint_velocity,
    const float* contact,
    unsigned part_base,
    unsigned joint_base,
    unsigned root_slot
) {
    float stack[16];
    int sp = 0;
    for (unsigned n = 0; n < count; ++n) {
        unsigned i = start + n;
        unsigned code = op_code[i];
        unsigned target = op_index[i];
        if (code == 0) stack[sp++] = op_a[i];
        else if (code == 1) stack[sp++] = time_seconds;
        else if (code == 2) stack[sp++] = position[root_slot * 3 + 1];
        else if (code == 3) stack[sp++] = velocity[root_slot * 3 + 0];
        else if (code == 4) stack[sp++] = velocity[root_slot * 3 + 1];
        else if (code == 5) stack[sp++] = velocity[root_slot * 3 + 2];
        else if (code == 6) stack[sp++] = angular_velocity[root_slot * 3 + 0];
        else if (code == 7) stack[sp++] = angular_velocity[root_slot * 3 + 1];
        else if (code == 8) stack[sp++] = angular_velocity[root_slot * 3 + 2];
        else if (code == 9) stack[sp++] = rotation[root_slot * 4 + 0];
        else if (code == 10) stack[sp++] = rotation[root_slot * 4 + 1];
        else if (code == 11) stack[sp++] = rotation[root_slot * 4 + 2];
        else if (code == 12) stack[sp++] = rotation[root_slot * 4 + 3];
        else if (code == 13) stack[sp++] = joint_angle[joint_base + target];
        else if (code == 14) stack[sp++] = joint_velocity[joint_base + target];
        else if (code == 15) stack[sp++] = contact[part_base + target];
        else if (code == 16 && sp >= 2) { float b = stack[--sp]; float a = stack[--sp]; stack[sp++] = a + b; }
        else if (code == 17 && sp >= 2) { float b = stack[--sp]; float a = stack[--sp]; stack[sp++] = a - b; }
        else if (code == 18 && sp >= 2) { float b = stack[--sp]; float a = stack[--sp]; stack[sp++] = a * b; }
        else if (code == 19 && sp >= 1) stack[sp - 1] = -stack[sp - 1];
        else if (code == 20 && sp >= 1) stack[sp - 1] = sinf(stack[sp - 1]);
        else if (code == 21 && sp >= 1) stack[sp - 1] = cosf(stack[sp - 1]);
        else if (code == 22 && sp >= 1) stack[sp - 1] = clampf(stack[sp - 1], op_a[i], op_b[i]);
        if (sp > 15) sp = 15;
    }
    return sp > 0 ? sane(stack[sp - 1]) : 0.0f;
}

extern "C" __global__ void simulate_creatures(
    const unsigned* part_count,
    const unsigned* joint_count,
    const unsigned* part_base_by_world,
    const unsigned* joint_base_by_world,
    const float* initial_position,
    const float* half_extents,
    const float* inv_mass,
    const float* inv_inertia,
    const float* friction,
    const unsigned* parent,
    const unsigned* child,
    const float* axis,
    const float* parent_anchor,
    const float* child_anchor,
    const float* rest_relative,
    const float* limit_min,
    const float* limit_max,
    const float* inertia,
    const float* biological_torque,
    const float* biological_power,
    const float* requested_torque,
    const unsigned* brain_start,
    const unsigned* brain_count,
    const unsigned* op_code,
    const unsigned* op_index,
    const float* op_a,
    const float* op_b,
    float* state_position,
    float* state_velocity,
    float* state_rotation,
    float* state_angular_velocity,
    float* state_contact,
    float* state_angle,
    float* state_angvel,
    float* state_target,
    float* joint_force,
    float* joint_torque,
    float* out_score,
    float* out_distance,
    float* out_speed,
    float* out_upright,
    float* out_stability,
    float* out_energy,
    unsigned* out_unstable,
    unsigned world_start,
    unsigned launch_world_count,
    float dt,
    unsigned steps,
    float gravity_y,
    float ground_y,
    float activation,
    float weight_distance,
    float weight_speed,
    float weight_upright,
    float weight_stability,
    float weight_energy
) {
    const unsigned GROUP_SIZE = 32;
    unsigned lane = threadIdx.x & 31u;
    unsigned local_world = blockIdx.x;
    if (local_world >= launch_world_count) return;

    unsigned world = world_start + local_world;
    const unsigned subgroup_mask = 0xFFFFFFFFu;

    unsigned pc = part_count[world];
    unsigned jc = joint_count[world];
    unsigned part_base = part_base_by_world[world];
    unsigned joint_base = joint_base_by_world[world];
    unsigned root_slot = part_base;

    for (unsigned p = lane; p < pc; p += GROUP_SIZE) {
        unsigned slot = part_base + p;
        state_position[slot * 3 + 0] = initial_position[slot * 3 + 0];
        state_position[slot * 3 + 1] = initial_position[slot * 3 + 1];
        state_position[slot * 3 + 2] = initial_position[slot * 3 + 2];
        state_velocity[slot * 3 + 0] = 0.0f;
        state_velocity[slot * 3 + 1] = 0.0f;
        state_velocity[slot * 3 + 2] = 0.0f;
        state_angular_velocity[slot * 3 + 0] = 0.0f;
        state_angular_velocity[slot * 3 + 1] = 0.0f;
        state_angular_velocity[slot * 3 + 2] = 0.0f;
        state_rotation[slot * 4 + 0] = 0.0f;
        state_rotation[slot * 4 + 1] = 0.0f;
        state_rotation[slot * 4 + 2] = 0.0f;
        state_rotation[slot * 4 + 3] = 1.0f;

        float tolerance = fmaxf(
            fmaxf(half_extents[slot * 3 + 0], half_extents[slot * 3 + 1]),
            half_extents[slot * 3 + 2]
        ) * 0.02f;
        float bottom_y = state_position[slot * 3 + 1] - half_extents[slot * 3 + 1];
        state_contact[slot] = bottom_y <= ground_y + fmaxf(tolerance, 1.0e-6f) ? 1.0f : 0.0f;
    }
    for (unsigned j = lane; j < jc; j += GROUP_SIZE) {
        unsigned js = joint_base + j;
        state_angle[js] = 0.0f;
        state_angvel[js] = 0.0f;
        state_target[js] = 0.0f;
        joint_force[js * 3 + 0] = 0.0f;
        joint_force[js * 3 + 1] = 0.0f;
        joint_force[js * 3 + 2] = 0.0f;
        joint_torque[js * 3 + 0] = 0.0f;
        joint_torque[js * 3 + 1] = 0.0f;
        joint_torque[js * 3 + 2] = 0.0f;
    }
    __syncwarp(subgroup_mask);

    float start_x = state_position[root_slot * 3 + 0];
    float start_z = state_position[root_slot * 3 + 2];
    float speed_sum = 0.0f;
    float upright_sum = lane == 0 ? 1.0f : 0.0f;
    float stability_sum = lane == 0 ? 1.0f : 0.0f;
    float local_motor_work = 0.0f;
    unsigned group_unstable = 0;

    for (unsigned step = 0; step < steps; ++step) {
        float time_seconds = (float)step * dt;

        for (unsigned p = lane; p < pc; p += GROUP_SIZE) {
            unsigned slot = part_base + p;
            state_velocity[slot * 3 + 1] += gravity_y * dt;
        }
        __syncwarp(subgroup_mask);

        for (unsigned j = lane; j < jc; j += GROUP_SIZE) {
            unsigned js = joint_base + j;
            float target = eval_brain(
                brain_start[js], brain_count[js], op_code, op_index, op_a, op_b,
                time_seconds, state_position, state_velocity, state_rotation,
                state_angular_velocity, state_angle, state_angvel, state_contact,
                part_base, joint_base, root_slot
            );
            state_target[js] = clampf(target, limit_min[js], limit_max[js]);
        }
        __syncwarp(subgroup_mask);

        for (unsigned j = lane; j < jc; j += GROUP_SIZE) {
            unsigned js = joint_base + j;
            unsigned ps = part_base + parent[js];
            unsigned cs = part_base + child[js];

            float pqx = state_rotation[ps * 4 + 0];
            float pqy = state_rotation[ps * 4 + 1];
            float pqz = state_rotation[ps * 4 + 2];
            float pqw = state_rotation[ps * 4 + 3];
            float cqx = state_rotation[cs * 4 + 0];
            float cqy = state_rotation[cs * 4 + 1];
            float cqz = state_rotation[cs * 4 + 2];
            float cqw = state_rotation[cs * 4 + 3];

            float wax, way, waz;
            rotate_vec(
                pqx, pqy, pqz, pqw,
                axis[js * 3 + 0], axis[js * 3 + 1], axis[js * 3 + 2],
                &wax, &way, &waz
            );

            float rel_wx = state_angular_velocity[cs * 3 + 0] - state_angular_velocity[ps * 3 + 0];
            float rel_wy = state_angular_velocity[cs * 3 + 1] - state_angular_velocity[ps * 3 + 1];
            float rel_wz = state_angular_velocity[cs * 3 + 2] - state_angular_velocity[ps * 3 + 2];
            float omega = rel_wx * wax + rel_wy * way + rel_wz * waz;
            state_angvel[js] = omega;

            float angle = clampf(state_angle[js] + omega * dt, limit_min[js], limit_max[js]);
            state_angle[js] = angle;

            float torque_cap = fminf(requested_torque[js], biological_torque[js]) * activation;
            float power_cap = biological_power[js] * activation;
            if (fabsf(omega) > 1.0e-4f && power_cap > 0.0f)
                torque_cap = fminf(torque_cap, power_cap / fabsf(omega));

            float span = fmaxf(fabsf(limit_max[js] - limit_min[js]), 1.0e-3f);
            float stiffness = torque_cap / fmaxf(span * 0.5f, 1.0e-3f);
            float damping = 2.0f * sqrtf(fmaxf(stiffness * inertia[js], 0.0f));
            float torque = clampf(
                stiffness * (state_target[js] - angle) - damping * omega,
                -torque_cap,
                torque_cap
            );
            local_motor_work += fabsf(torque * omega) * dt;

            joint_torque[js * 3 + 0] = wax * torque;
            joint_torque[js * 3 + 1] = way * torque;
            joint_torque[js * 3 + 2] = waz * torque;

            float pax, pay, paz;
            float cax, cay, caz;
            rotate_vec(
                pqx, pqy, pqz, pqw,
                parent_anchor[js * 3 + 0], parent_anchor[js * 3 + 1], parent_anchor[js * 3 + 2],
                &pax, &pay, &paz
            );
            rotate_vec(
                cqx, cqy, cqz, cqw,
                child_anchor[js * 3 + 0], child_anchor[js * 3 + 1], child_anchor[js * 3 + 2],
                &cax, &cay, &caz
            );

            float ppx = state_position[ps * 3 + 0] + pax;
            float ppy = state_position[ps * 3 + 1] + pay;
            float ppz = state_position[ps * 3 + 2] + paz;
            float cpx = state_position[cs * 3 + 0] + cax;
            float cpy = state_position[cs * 3 + 1] + cay;
            float cpz = state_position[cs * 3 + 2] + caz;

            float pwx = state_angular_velocity[ps * 3 + 0];
            float pwy = state_angular_velocity[ps * 3 + 1];
            float pwz = state_angular_velocity[ps * 3 + 2];
            float cwx = state_angular_velocity[cs * 3 + 0];
            float cwy = state_angular_velocity[cs * 3 + 1];
            float cwz = state_angular_velocity[cs * 3 + 2];

            float pvx = state_velocity[ps * 3 + 0] + (pwy * paz - pwz * pay);
            float pvy = state_velocity[ps * 3 + 1] + (pwz * pax - pwx * paz);
            float pvz = state_velocity[ps * 3 + 2] + (pwx * pay - pwy * pax);
            float cvx = state_velocity[cs * 3 + 0] + (cwy * caz - cwz * cay);
            float cvy = state_velocity[cs * 3 + 1] + (cwz * cax - cwx * caz);
            float cvz = state_velocity[cs * 3 + 2] + (cwx * cay - cwy * cax);

            float error_x = ppx - cpx;
            float error_y = ppy - cpy;
            float error_z = ppz - cpz;
            float rel_vx = cvx - pvx;
            float rel_vy = cvy - pvy;
            float rel_vz = cvz - pvz;

            float reduced_mass = 1.0f / fmaxf(inv_mass[ps] + inv_mass[cs], 1.0e-12f);
            float constraint_omega = fminf(125.663706f, 0.35f / fmaxf(dt, 1.0e-6f));
            float k_linear = reduced_mass * constraint_omega * constraint_omega;
            float d_linear = 2.0f * reduced_mass * constraint_omega;
            float fx = error_x * k_linear - rel_vx * d_linear;
            float fy = error_y * k_linear - rel_vy * d_linear;
            float fz = error_z * k_linear - rel_vz * d_linear;

            joint_force[js * 3 + 0] = fx;
            joint_force[js * 3 + 1] = fy;
            joint_force[js * 3 + 2] = fz;
        }
        __syncwarp(subgroup_mask);

        int lane_unstable = 0;
        for (unsigned p = lane; p < pc; p += GROUP_SIZE) {
            unsigned slot = part_base + p;
            float fx = 0.0f;
            float fy = 0.0f;
            float fz = 0.0f;
            float tx = 0.0f;
            float ty = 0.0f;
            float tz = 0.0f;

            for (unsigned j = 0; j < jc; ++j) {
                unsigned js = joint_base + j;
                bool is_parent = parent[js] == p;
                bool is_child = child[js] == p;
                if (!is_parent && !is_child) continue;

                float jfx = joint_force[js * 3 + 0];
                float jfy = joint_force[js * 3 + 1];
                float jfz = joint_force[js * 3 + 2];
                float jtx = joint_torque[js * 3 + 0];
                float jty = joint_torque[js * 3 + 1];
                float jtz = joint_torque[js * 3 + 2];

                float qx = state_rotation[slot * 4 + 0];
                float qy = state_rotation[slot * 4 + 1];
                float qz = state_rotation[slot * 4 + 2];
                float qw = state_rotation[slot * 4 + 3];
                float rx, ry, rz;

                if (is_parent) {
                    fx -= jfx; fy -= jfy; fz -= jfz;
                    tx -= jtx; ty -= jty; tz -= jtz;
                    rotate_vec(
                        qx, qy, qz, qw,
                        parent_anchor[js * 3 + 0], parent_anchor[js * 3 + 1], parent_anchor[js * 3 + 2],
                        &rx, &ry, &rz
                    );
                    tx += ry * (-jfz) - rz * (-jfy);
                    ty += rz * (-jfx) - rx * (-jfz);
                    tz += rx * (-jfy) - ry * (-jfx);
                }
                if (is_child) {
                    fx += jfx; fy += jfy; fz += jfz;
                    tx += jtx; ty += jty; tz += jtz;
                    rotate_vec(
                        qx, qy, qz, qw,
                        child_anchor[js * 3 + 0], child_anchor[js * 3 + 1], child_anchor[js * 3 + 2],
                        &rx, &ry, &rz
                    );
                    tx += ry * jfz - rz * jfy;
                    ty += rz * jfx - rx * jfz;
                    tz += rx * jfy - ry * jfx;
                }
            }

            float linear_impulse = inv_mass[slot] * dt;
            state_velocity[slot * 3 + 0] += fx * linear_impulse;
            state_velocity[slot * 3 + 1] += fy * linear_impulse;
            state_velocity[slot * 3 + 2] += fz * linear_impulse;

            float angular_impulse = inv_inertia[slot] * dt;
            state_angular_velocity[slot * 3 + 0] += tx * angular_impulse;
            state_angular_velocity[slot * 3 + 1] += ty * angular_impulse;
            state_angular_velocity[slot * 3 + 2] += tz * angular_impulse;

            state_position[slot * 3 + 0] += state_velocity[slot * 3 + 0] * dt;
            state_position[slot * 3 + 1] += state_velocity[slot * 3 + 1] * dt;
            state_position[slot * 3 + 2] += state_velocity[slot * 3 + 2] * dt;
            integrate_quat(
                state_rotation,
                slot,
                state_angular_velocity[slot * 3 + 0],
                state_angular_velocity[slot * 3 + 1],
                state_angular_velocity[slot * 3 + 2],
                dt
            );

            state_contact[slot] = 0.0f;
            float floor_y = ground_y + projected_half_height(state_rotation, half_extents, slot);
            if (state_position[slot * 3 + 1] < floor_y) {
                state_position[slot * 3 + 1] = floor_y;
                if (state_velocity[slot * 3 + 1] < 0.0f)
                    state_velocity[slot * 3 + 1] = 0.0f;
                state_contact[slot] = 1.0f;

                float vx = state_velocity[slot * 3 + 0];
                float vz = state_velocity[slot * 3 + 2];
                float horizontal_speed = sqrtf(vx * vx + vz * vz);
                if (horizontal_speed > 1.0e-8f) {
                    float max_delta = friction[slot] * fabsf(gravity_y) * dt;
                    float scale = fmaxf(0.0f, 1.0f - max_delta / horizontal_speed);
                    state_velocity[slot * 3 + 0] *= scale;
                    state_velocity[slot * 3 + 2] *= scale;
                }

                float angular_drag = fmaxf(0.0f, 1.0f - friction[slot] * 2.0f * dt);
                state_angular_velocity[slot * 3 + 0] *= angular_drag;
                state_angular_velocity[slot * 3 + 1] *= angular_drag;
                state_angular_velocity[slot * 3 + 2] *= angular_drag;
            }

            float vx = state_velocity[slot * 3 + 0];
            float vy = state_velocity[slot * 3 + 1];
            float vz = state_velocity[slot * 3 + 2];
            float wx = state_angular_velocity[slot * 3 + 0];
            float wy = state_angular_velocity[slot * 3 + 1];
            float wz = state_angular_velocity[slot * 3 + 2];
            float x = state_position[slot * 3 + 0];
            float y = state_position[slot * 3 + 1];
            float z = state_position[slot * 3 + 2];
            float linear_speed_sq = vx * vx + vy * vy + vz * vz;
            float angular_speed_sq = wx * wx + wy * wy + wz * wz;
            if (
                !(vx == vx) || !(vy == vy) || !(vz == vz)
                || !(wx == wx) || !(wy == wy) || !(wz == wz)
                || !(x == x) || !(y == y) || !(z == z)
                || !(linear_speed_sq == linear_speed_sq)
                || !(angular_speed_sq == angular_speed_sq)
                || linear_speed_sq > 10000.0f
                || angular_speed_sq > 1000000.0f
            ) {
                lane_unstable = 1;
            }
        }
        if (__any_sync(subgroup_mask, lane_unstable)) group_unstable = 1;
        __syncwarp(subgroup_mask);

        if (group_unstable) break;

        if (lane == 0) {
            float rvx = state_velocity[root_slot * 3 + 0];
            float rvz = state_velocity[root_slot * 3 + 2];
            speed_sum += sqrtf(rvx * rvx + rvz * rvz);

            float qx = state_rotation[root_slot * 4 + 0];
            float qz = state_rotation[root_slot * 4 + 2];
            float up_y = 1.0f - 2.0f * (qx * qx + qz * qz);
            upright_sum += clampf(up_y, 0.0f, 1.0f);

            float wx = state_angular_velocity[root_slot * 3 + 0];
            float wy = state_angular_velocity[root_slot * 3 + 1];
            float wz = state_angular_velocity[root_slot * 3 + 2];
            float angular_speed = sqrtf(wx * wx + wy * wy + wz * wz);
            stability_sum += 1.0f / (1.0f + angular_speed);
        }
        __syncwarp(subgroup_mask);
    }

    for (unsigned offset = GROUP_SIZE / 2; offset > 0; offset >>= 1)
        local_motor_work += __shfl_down_sync(
            subgroup_mask, local_motor_work, offset, GROUP_SIZE
        );

    if (lane != 0) return;

    if (group_unstable) {
        out_unstable[world] = 1;
        out_score[world] = -1.0e30f;
        out_distance[world] = 0.0f;
        out_speed[world] = 0.0f;
        out_upright[world] = 0.0f;
        out_stability[world] = 0.0f;
        out_energy[world] = 0.0f;
        return;
    }

    float dx = state_position[root_slot * 3 + 0] - start_x;
    float dz = state_position[root_slot * 3 + 2] - start_z;
    float distance = sqrtf(dx * dx + dz * dz);
    float denom = (float)steps + 1.0f;
    float average_speed = speed_sum / denom;
    float upright = upright_sum / denom;
    float stability = stability_sum / denom;
    float simulated_seconds = fmaxf((float)steps * dt, 1.0e-8f);
    float energy = local_motor_work / simulated_seconds;
    float score = weight_distance * distance
        + weight_speed * average_speed
        + weight_upright * upright
        + weight_stability * stability
        + weight_energy * energy;

    out_unstable[world] = 0;
    out_score[world] = score;
    out_distance[world] = distance;
    out_speed[world] = average_speed;
    out_upright[world] = upright;
    out_stability[world] = stability;
    out_energy[world] = energy;
}
"#;
}
