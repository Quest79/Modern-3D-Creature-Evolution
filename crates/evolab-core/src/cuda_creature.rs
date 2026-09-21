use crate::{
    AcceleratorConfig, CreatureGenome, ExecutionPerformance, FitnessConfig, FitnessResult,
    SimulationConfig, TerrainKind, discover_cuda_devices, schedule_gpu_work,
};

#[cfg(windows)]
use crate::{DeviceWorkAssignment, Expression, FitnessMetrics, SensorKind, legacy_expression};

#[derive(Clone, Debug)]
pub struct CudaCreatureBatchResult {
    pub fitness: Vec<FitnessResult>,
    pub execution: ExecutionPerformance,
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

    let devices = discover_cuda_devices()?;
    if devices.is_empty() {
        return Err("no CUDA devices were found".into());
    }
    let selected = accelerator.selected_gpu_ids(&devices)?;
    if selected.is_empty() {
        return Err("no CUDA devices were selected".into());
    }

    let assignments = schedule_gpu_work(genomes.len(), &selected)?;
    platform::run_batch(
        genomes,
        simulation,
        fitness,
        accelerator,
        &devices,
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
    max_parts: usize,
    max_joints: usize,
    part_count: Vec<u32>,
    joint_count: Vec<u32>,
    initial_position: Vec<f32>,
    half_extents: Vec<f32>,
    mass: Vec<f32>,
    inv_mass: Vec<f32>,
    friction: Vec<f32>,
    parent: Vec<u32>,
    child: Vec<u32>,
    axis: Vec<f32>,
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
        max_parts: usize,
        max_joints: usize,
    ) -> Result<Self, String> {
        let world_count = genomes.len();
        let part_slots = world_count * max_parts;
        let joint_slots = world_count * max_joints;
        let mut packed = Self {
            world_count,
            max_parts,
            max_joints,
            part_count: vec![0; world_count],
            joint_count: vec![0; world_count],
            initial_position: vec![0.0; part_slots * 3],
            half_extents: vec![0.0; part_slots * 3],
            mass: vec![0.0; part_slots],
            inv_mass: vec![0.0; part_slots],
            friction: vec![0.0; part_slots],
            parent: vec![0; joint_slots],
            child: vec![0; joint_slots],
            axis: vec![0.0; joint_slots * 3],
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

        for (world_index, genome) in genomes.iter().enumerate() {
            packed.part_count[world_index] = genome.segments.len() as u32;
            packed.joint_count[world_index] = genome.joints.len() as u32;

            let mut segment_index = std::collections::HashMap::new();
            for (index, segment) in genome.segments.iter().enumerate() {
                segment_index.insert(segment.id, index as u32);
                let slot = world_index * max_parts + index;
                for axis in 0..3 {
                    packed.initial_position[slot * 3 + axis] = segment.initial_position[axis];
                    packed.half_extents[slot * 3 + axis] = segment.half_extents[axis];
                }
                let mass = segment.mass_kg().max(1.0e-12);
                packed.mass[slot] = mass;
                packed.inv_mass[slot] = mass.recip();
                packed.friction[slot] = segment.friction;
            }

            let mut joint_index = std::collections::HashMap::new();
            for (index, joint) in genome.joints.iter().enumerate() {
                joint_index.insert(joint.child_id, index as u32);
            }

            for (index, joint) in genome.joints.iter().enumerate() {
                let slot = world_index * max_joints + index;
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
        marker::PhantomData,
        mem::{size_of, transmute},
        os::windows::ffi::OsStrExt,
        path::{Path, PathBuf},
        ptr::{null, null_mut},
        sync::{Mutex, OnceLock},
        thread,
        time::Instant,
    };

    use crate::{
        AcceleratorConfig, AcceleratorMode, CreatureGenome, CudaDeviceInfo, DevicePerformance,
        DeviceWorkAssignment, ExecutionPerformance, FitnessConfig, FitnessMetrics, FitnessResult,
        SimulationConfig, ThroughputMode,
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
    type CuCtxSynchronize = unsafe extern "system" fn() -> CuResult;
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
        ctx_synchronize: CuCtxSynchronize,
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
                ctx_synchronize: load!("cuCtxSynchronize", CuCtxSynchronize),
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

    struct ContextGuard<'a> {
        api: &'a CudaApi,
        context: CuContext,
    }

    impl Drop for ContextGuard<'_> {
        fn drop(&mut self) {
            if !self.context.is_null() {
                unsafe {
                    let _ = (self.api.ctx_destroy)(self.context);
                }
            }
        }
    }

    struct ModuleGuard<'a> {
        api: &'a CudaApi,
        module: CuModule,
    }

    impl Drop for ModuleGuard<'_> {
        fn drop(&mut self) {
            if !self.module.is_null() {
                unsafe {
                    let _ = (self.api.module_unload)(self.module);
                }
            }
        }
    }

    struct DeviceBuffer<'a, T> {
        api: &'a CudaApi,
        pointer: CuDevicePtr,
        len: usize,
        _marker: PhantomData<T>,
    }

    impl<'a, T> DeviceBuffer<'a, T> {
        fn allocate(api: &'a CudaApi, len: usize) -> Result<Self, String> {
            let bytes = len.max(1) * size_of::<T>();
            let mut pointer = 0;
            check_cuda(
                unsafe { (api.mem_alloc)(&mut pointer, bytes) },
                "cuMemAlloc",
            )?;
            Ok(Self {
                api,
                pointer,
                len,
                _marker: PhantomData,
            })
        }

        fn copy_from(api: &'a CudaApi, values: &[T]) -> Result<Self, String> {
            let buffer = Self::allocate(api, values.len())?;
            if !values.is_empty() {
                check_cuda(
                    unsafe {
                        (api.memcpy_htod)(
                            buffer.pointer,
                            values.as_ptr().cast::<c_void>(),
                            values.len() * size_of::<T>(),
                        )
                    },
                    "cuMemcpyHtoD",
                )?;
            }
            Ok(buffer)
        }

        fn copy_to(&self, values: &mut [T]) -> Result<(), String> {
            if values.len() != self.len {
                return Err("CUDA output buffer length mismatch".into());
            }
            if !values.is_empty() {
                check_cuda(
                    unsafe {
                        (self.api.memcpy_dtoh)(
                            values.as_mut_ptr().cast::<c_void>(),
                            self.pointer,
                            values.len() * size_of::<T>(),
                        )
                    },
                    "cuMemcpyDtoH",
                )?;
            }
            Ok(())
        }
    }

    impl<T> Drop for DeviceBuffer<'_, T> {
        fn drop(&mut self) {
            if self.pointer != 0 {
                unsafe {
                    let _ = (self.api.mem_free)(self.pointer);
                }
            }
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
            execution: ExecutionPerformance {
                requested_mode: accelerator.mode,
                actual_mode: AcceleratorMode::Cuda,
                cpu_items: 0,
                gpu_items: genomes.len(),
                fallback_items: 0,
                wall_seconds,
                items_per_second: genomes.len() as f64 / wall_seconds,
                physics_steps_per_second: physics_steps as f64 / wall_seconds,
                devices: device_stats,
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
        let api = CudaApi::load()?;
        let mut device = 0;
        check_cuda(
            unsafe { (api.device_get)(&mut device, device_info.id as i32) },
            "cuDeviceGet",
        )?;
        let mut context = null_mut();
        check_cuda(
            unsafe { (api.ctx_create)(&mut context, 0, device) },
            "cuCtxCreate",
        )?;
        let _context_guard = ContextGuard { api: &api, context };

        let ptx = compile_kernel(
            device_info.compute_capability_major,
            device_info.compute_capability_minor,
            accelerator.throughput_mode,
        )?;
        let mut module = null_mut();
        check_cuda(
            unsafe { (api.module_load_data)(&mut module, ptx.as_ptr().cast::<c_void>()) },
            "cuModuleLoadData",
        )?;
        let _module_guard = ModuleGuard { api: &api, module };
        let kernel_name = CString::new("simulate_creatures").expect("static kernel name");
        let mut function = null_mut();
        check_cuda(
            unsafe { (api.module_get_function)(&mut function, module, kernel_name.as_ptr()) },
            "cuModuleGetFunction",
        )?;

        let started = Instant::now();
        let mut all_results = Vec::with_capacity(genomes.len());
        let batch_size = accelerator.batch_size.max(1);
        let mut offset = 0usize;
        while offset < genomes.len() {
            let end = (offset + batch_size).min(genomes.len());
            let chunk = &genomes[offset..end];
            let chunk_results = run_chunk(&api, function, chunk, simulation, fitness, accelerator)?;
            all_results.extend(
                chunk_results
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
            },
        })
    }

    fn run_chunk(
        api: &CudaApi,
        function: CuFunction,
        genomes: &[CreatureGenome],
        simulation: &SimulationConfig,
        fitness: &FitnessConfig,
        accelerator: &AcceleratorConfig,
    ) -> Result<Vec<FitnessResult>, String> {
        let packed =
            PackedCreatureBatch::pack(genomes, accelerator.max_parts, accelerator.max_joints)?;
        let world_count = packed.world_count;
        let part_slots = world_count * packed.max_parts;
        let joint_slots = world_count * packed.max_joints;

        let d_part_count = DeviceBuffer::copy_from(api, &packed.part_count)?;
        let d_joint_count = DeviceBuffer::copy_from(api, &packed.joint_count)?;
        let d_initial_position = DeviceBuffer::copy_from(api, &packed.initial_position)?;
        let d_half_extents = DeviceBuffer::copy_from(api, &packed.half_extents)?;
        let d_mass = DeviceBuffer::copy_from(api, &packed.mass)?;
        let d_inv_mass = DeviceBuffer::copy_from(api, &packed.inv_mass)?;
        let d_friction = DeviceBuffer::copy_from(api, &packed.friction)?;
        let d_parent = DeviceBuffer::copy_from(api, &packed.parent)?;
        let d_child = DeviceBuffer::copy_from(api, &packed.child)?;
        let d_axis = DeviceBuffer::copy_from(api, &packed.axis)?;
        let d_rest_relative = DeviceBuffer::copy_from(api, &packed.rest_relative)?;
        let d_limit_min = DeviceBuffer::copy_from(api, &packed.limit_min)?;
        let d_limit_max = DeviceBuffer::copy_from(api, &packed.limit_max)?;
        let d_inertia = DeviceBuffer::copy_from(api, &packed.inertia)?;
        let d_biological_torque = DeviceBuffer::copy_from(api, &packed.biological_torque)?;
        let d_biological_power = DeviceBuffer::copy_from(api, &packed.biological_power)?;
        let d_requested_torque = DeviceBuffer::copy_from(api, &packed.requested_torque)?;
        let d_brain_start = DeviceBuffer::copy_from(api, &packed.brain_start)?;
        let d_brain_count = DeviceBuffer::copy_from(api, &packed.brain_count)?;
        let d_op_code = DeviceBuffer::copy_from(api, &packed.op_code)?;
        let d_op_index = DeviceBuffer::copy_from(api, &packed.op_index)?;
        let d_op_a = DeviceBuffer::copy_from(api, &packed.op_a)?;
        let d_op_b = DeviceBuffer::copy_from(api, &packed.op_b)?;

        let d_state_position = DeviceBuffer::<f32>::allocate(api, part_slots * 3)?;
        let d_state_velocity = DeviceBuffer::<f32>::allocate(api, part_slots * 3)?;
        let d_state_contact = DeviceBuffer::<f32>::allocate(api, part_slots)?;
        let d_state_angle = DeviceBuffer::<f32>::allocate(api, joint_slots)?;
        let d_state_angvel = DeviceBuffer::<f32>::allocate(api, joint_slots)?;

        let d_out_score = DeviceBuffer::<f32>::allocate(api, world_count)?;
        let d_out_distance = DeviceBuffer::<f32>::allocate(api, world_count)?;
        let d_out_speed = DeviceBuffer::<f32>::allocate(api, world_count)?;
        let d_out_upright = DeviceBuffer::<f32>::allocate(api, world_count)?;
        let d_out_stability = DeviceBuffer::<f32>::allocate(api, world_count)?;
        let d_out_energy = DeviceBuffer::<f32>::allocate(api, world_count)?;
        let d_out_unstable = DeviceBuffer::<u32>::allocate(api, world_count)?;

        let mut p_part_count = d_part_count.pointer;
        let mut p_joint_count = d_joint_count.pointer;
        let mut p_initial_position = d_initial_position.pointer;
        let mut p_half_extents = d_half_extents.pointer;
        let mut p_mass = d_mass.pointer;
        let mut p_inv_mass = d_inv_mass.pointer;
        let mut p_friction = d_friction.pointer;
        let mut p_parent = d_parent.pointer;
        let mut p_child = d_child.pointer;
        let mut p_axis = d_axis.pointer;
        let mut p_rest_relative = d_rest_relative.pointer;
        let mut p_limit_min = d_limit_min.pointer;
        let mut p_limit_max = d_limit_max.pointer;
        let mut p_inertia = d_inertia.pointer;
        let mut p_biological_torque = d_biological_torque.pointer;
        let mut p_biological_power = d_biological_power.pointer;
        let mut p_requested_torque = d_requested_torque.pointer;
        let mut p_brain_start = d_brain_start.pointer;
        let mut p_brain_count = d_brain_count.pointer;
        let mut p_op_code = d_op_code.pointer;
        let mut p_op_index = d_op_index.pointer;
        let mut p_op_a = d_op_a.pointer;
        let mut p_op_b = d_op_b.pointer;
        let mut p_state_position = d_state_position.pointer;
        let mut p_state_velocity = d_state_velocity.pointer;
        let mut p_state_contact = d_state_contact.pointer;
        let mut p_state_angle = d_state_angle.pointer;
        let mut p_state_angvel = d_state_angvel.pointer;
        let mut p_out_score = d_out_score.pointer;
        let mut p_out_distance = d_out_distance.pointer;
        let mut p_out_speed = d_out_speed.pointer;
        let mut p_out_upright = d_out_upright.pointer;
        let mut p_out_stability = d_out_stability.pointer;
        let mut p_out_energy = d_out_energy.pointer;
        let mut p_out_unstable = d_out_unstable.pointer;

        let mut world_count_arg = world_count as u32;
        let mut max_parts_arg = packed.max_parts as u32;
        let mut max_joints_arg = packed.max_joints as u32;
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

        let mut params = [
            (&mut p_part_count as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_joint_count as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_initial_position as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_half_extents as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_mass as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_inv_mass as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_friction as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_parent as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_child as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_axis as *mut CuDevicePtr).cast::<c_void>(),
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
            (&mut p_state_contact as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_state_angle as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_state_angvel as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_out_score as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_out_distance as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_out_speed as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_out_upright as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_out_stability as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_out_energy as *mut CuDevicePtr).cast::<c_void>(),
            (&mut p_out_unstable as *mut CuDevicePtr).cast::<c_void>(),
            (&mut world_count_arg as *mut u32).cast::<c_void>(),
            (&mut max_parts_arg as *mut u32).cast::<c_void>(),
            (&mut max_joints_arg as *mut u32).cast::<c_void>(),
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

        let block_size = 128_u32;
        let grid_size = (world_count as u32).div_ceil(block_size);
        check_cuda(
            unsafe {
                (api.launch_kernel)(
                    function,
                    grid_size,
                    1,
                    1,
                    block_size,
                    1,
                    1,
                    0,
                    null_mut(),
                    params.as_mut_ptr(),
                    null_mut(),
                )
            },
            "cuLaunchKernel(simulate_creatures)",
        )?;
        check_cuda(unsafe { (api.ctx_synchronize)() }, "cuCtxSynchronize")?;

        let mut score = vec![0.0; world_count];
        let mut distance = vec![0.0; world_count];
        let mut speed = vec![0.0; world_count];
        let mut upright = vec![0.0; world_count];
        let mut stability = vec![0.0; world_count];
        let mut energy = vec![0.0; world_count];
        let mut unstable = vec![0_u32; world_count];
        d_out_score.copy_to(&mut score)?;
        d_out_distance.copy_to(&mut distance)?;
        d_out_speed.copy_to(&mut speed)?;
        d_out_upright.copy_to(&mut upright)?;
        d_out_stability.copy_to(&mut stability)?;
        d_out_energy.copy_to(&mut energy)?;
        d_out_unstable.copy_to(&mut unstable)?;

        Ok((0..world_count)
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
            .collect())
    }

    const CUDA_CREATURE_SOURCE: &str = r#"
extern "C" __device__ float clampf(float x, float lo, float hi) {
    return fminf(fmaxf(x, lo), hi);
}

extern "C" __device__ float sane(float x) {
    if (!(x == x) || fabsf(x) > 1000.0f) return 0.0f;
    return clampf(x, -1000.0f, 1000.0f);
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
    const float* joint_angle,
    const float* joint_velocity,
    const float* contact,
    unsigned part_base,
    unsigned joint_base,
    unsigned root_slot
) {
    float stack[64];
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
        else if (code == 6 || code == 7 || code == 8) stack[sp++] = 0.0f;
        else if (code == 9 || code == 10 || code == 11) stack[sp++] = 0.0f;
        else if (code == 12) stack[sp++] = 1.0f;
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
        if (sp > 63) sp = 63;
    }
    return sp > 0 ? sane(stack[sp - 1]) : 0.0f;
}

extern "C" __global__ void simulate_creatures(
    const unsigned* part_count,
    const unsigned* joint_count,
    const float* initial_position,
    const float* half_extents,
    const float* mass,
    const float* inv_mass,
    const float* friction,
    const unsigned* parent,
    const unsigned* child,
    const float* axis,
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
    float* state_contact,
    float* state_angle,
    float* state_angvel,
    float* out_score,
    float* out_distance,
    float* out_speed,
    float* out_upright,
    float* out_stability,
    float* out_energy,
    unsigned* out_unstable,
    unsigned world_count,
    unsigned max_parts,
    unsigned max_joints,
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
    unsigned world = blockIdx.x * blockDim.x + threadIdx.x;
    if (world >= world_count) return;

    unsigned pc = part_count[world];
    unsigned jc = joint_count[world];
    unsigned part_base = world * max_parts;
    unsigned joint_base = world * max_joints;
    unsigned root_slot = part_base;

    for (unsigned p = 0; p < pc; ++p) {
        unsigned slot = part_base + p;
        for (unsigned c = 0; c < 3; ++c) {
            state_position[slot * 3 + c] = initial_position[slot * 3 + c];
            state_velocity[slot * 3 + c] = 0.0f;
        }
        state_contact[slot] = 0.0f;
    }
    for (unsigned j = 0; j < jc; ++j) {
        state_angle[joint_base + j] = 0.0f;
        state_angvel[joint_base + j] = 0.0f;
    }

    float start_x = state_position[root_slot * 3 + 0];
    float start_z = state_position[root_slot * 3 + 2];
    float start_root_height = fmaxf(
        state_position[root_slot * 3 + 1] - ground_y,
        half_extents[root_slot * 3 + 1]
    );
    float speed_sum = 0.0f;
    float upright_sum = 0.0f;
    float stability_sum = 0.0f;
    float motor_work = 0.0f;
    unsigned unstable = 0;

    for (unsigned step = 0; step < steps; ++step) {
        float time_seconds = (float)step * dt;

        for (unsigned p = 0; p < pc; ++p) {
            unsigned slot = part_base + p;
            state_velocity[slot * 3 + 1] += gravity_y * dt;
            state_contact[slot] = 0.0f;
        }

        for (unsigned j = 0; j < jc; ++j) {
            unsigned js = joint_base + j;
            unsigned pi = parent[js];
            unsigned ci = child[js];
            unsigned ps = part_base + pi;
            unsigned cs = part_base + ci;

            float target = eval_brain(
                brain_start[js], brain_count[js], op_code, op_index, op_a, op_b,
                time_seconds, state_position, state_velocity, state_angle, state_angvel,
                state_contact, part_base, joint_base, root_slot
            );
            target = clampf(target, limit_min[js], limit_max[js]);

            float torque_cap = fminf(requested_torque[js], biological_torque[js]) * activation;
            float angle = state_angle[js];
            float omega = state_angvel[js];
            float power_cap = biological_power[js] * activation;
            if (fabsf(omega) > 1.0e-4f && power_cap > 0.0f)
                torque_cap = fminf(torque_cap, power_cap / fabsf(omega));

            float span = fmaxf(fabsf(limit_max[js] - limit_min[js]), 1.0e-3f);
            float stiffness = torque_cap / fmaxf(span * 0.5f, 1.0e-3f);
            float damping = 2.0f * sqrtf(fmaxf(stiffness * inertia[js], 0.0f));
            float torque = clampf(stiffness * (target - angle) - damping * omega, -torque_cap, torque_cap);
            float angular_accel = torque / fmaxf(inertia[js], 1.0e-12f);
            omega += angular_accel * dt;
            angle += omega * dt;
            if (angle < limit_min[js]) { angle = limit_min[js]; if (omega < 0.0f) omega = 0.0f; }
            if (angle > limit_max[js]) { angle = limit_max[js]; if (omega > 0.0f) omega = 0.0f; }
            state_angle[js] = angle;
            state_angvel[js] = omega;
            motor_work += fabsf(torque * omega) * dt;

            float ax = axis[js * 3 + 0];
            float ay = axis[js * 3 + 1];
            float az = axis[js * 3 + 2];
            float rx = rest_relative[js * 3 + 0];
            float ry = rest_relative[js * 3 + 1];
            float rz = rest_relative[js * 3 + 2];
            float s = sinf(angle);
            float c = cosf(angle);
            float dot = ax * rx + ay * ry + az * rz;
            float cross_x = ay * rz - az * ry;
            float cross_y = az * rx - ax * rz;
            float cross_z = ax * ry - ay * rx;
            float one_minus_c = 1.0f - c;
            float desired_x = rx * c + cross_x * s + ax * dot * one_minus_c;
            float desired_y = ry * c + cross_y * s + ay * dot * one_minus_c;
            float desired_z = rz * c + cross_z * s + az * dot * one_minus_c;

            float actual_x = state_position[cs * 3 + 0] - state_position[ps * 3 + 0];
            float actual_y = state_position[cs * 3 + 1] - state_position[ps * 3 + 1];
            float actual_z = state_position[cs * 3 + 2] - state_position[ps * 3 + 2];
            float error_x = desired_x - actual_x;
            float error_y = desired_y - actual_y;
            float error_z = desired_z - actual_z;
            float rel_vx = state_velocity[cs * 3 + 0] - state_velocity[ps * 3 + 0];
            float rel_vy = state_velocity[cs * 3 + 1] - state_velocity[ps * 3 + 1];
            float rel_vz = state_velocity[cs * 3 + 2] - state_velocity[ps * 3 + 2];

            float lever = fmaxf(sqrtf(rx * rx + ry * ry + rz * rz), 1.0e-4f);
            float max_force = torque_cap / lever;
            float k_linear = max_force / fmaxf(lever * 0.25f, 1.0e-5f);
            float reduced_mass = 1.0f / fmaxf(inv_mass[ps] + inv_mass[cs], 1.0e-12f);
            float d_linear = 2.0f * sqrtf(fmaxf(k_linear * reduced_mass, 0.0f));
            float fx = error_x * k_linear - rel_vx * d_linear;
            float fy = error_y * k_linear - rel_vy * d_linear;
            float fz = error_z * k_linear - rel_vz * d_linear;
            float force_mag = sqrtf(fx * fx + fy * fy + fz * fz);
            if (force_mag > max_force && force_mag > 1.0e-8f) {
                float scale = max_force / force_mag;
                fx *= scale; fy *= scale; fz *= scale;
            }

            float parent_impulse = inv_mass[ps] * dt;
            float child_impulse = inv_mass[cs] * dt;
            state_velocity[ps * 3 + 0] -= fx * parent_impulse;
            state_velocity[ps * 3 + 1] -= fy * parent_impulse;
            state_velocity[ps * 3 + 2] -= fz * parent_impulse;
            state_velocity[cs * 3 + 0] += fx * child_impulse;
            state_velocity[cs * 3 + 1] += fy * child_impulse;
            state_velocity[cs * 3 + 2] += fz * child_impulse;
        }

        for (unsigned p = 0; p < pc; ++p) {
            unsigned slot = part_base + p;
            state_position[slot * 3 + 0] += state_velocity[slot * 3 + 0] * dt;
            state_position[slot * 3 + 1] += state_velocity[slot * 3 + 1] * dt;
            state_position[slot * 3 + 2] += state_velocity[slot * 3 + 2] * dt;

            float floor_y = ground_y + half_extents[slot * 3 + 1];
            if (state_position[slot * 3 + 1] < floor_y) {
                state_position[slot * 3 + 1] = floor_y;
                if (state_velocity[slot * 3 + 1] < 0.0f) state_velocity[slot * 3 + 1] = 0.0f;
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
            }

            float vx = state_velocity[slot * 3 + 0];
            float vy = state_velocity[slot * 3 + 1];
            float vz = state_velocity[slot * 3 + 2];
            float x = state_position[slot * 3 + 0];
            float y = state_position[slot * 3 + 1];
            float z = state_position[slot * 3 + 2];
            if (!(vx == vx) || !(vy == vy) || !(vz == vz) || !(x == x) || !(y == y) || !(z == z)
                || fabsf(vx) > 10000.0f || fabsf(vy) > 10000.0f || fabsf(vz) > 10000.0f) {
                unstable = 1;
            }
        }

        float rvx = state_velocity[root_slot * 3 + 0];
        float rvz = state_velocity[root_slot * 3 + 2];
        speed_sum += sqrtf(rvx * rvx + rvz * rvz);
        float root_height = state_position[root_slot * 3 + 1] - ground_y;
        upright_sum += clampf(root_height / fmaxf(start_root_height, 1.0e-6f), 0.0f, 1.0f);

        float omega_sum = 0.0f;
        for (unsigned j = 0; j < jc; ++j) omega_sum += fabsf(state_angvel[joint_base + j]);
        float omega_mean = jc > 0 ? omega_sum / (float)jc : 0.0f;
        stability_sum += 1.0f / (1.0f + omega_mean);

        if (unstable) break;
    }

    if (unstable) {
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
    float denom = steps > 0 ? (float)steps : 1.0f;
    float average_speed = speed_sum / denom;
    float upright = upright_sum / denom;
    float stability = stability_sum / denom;
    float simulated_seconds = fmaxf((float)steps * dt, 1.0e-8f);
    float energy = motor_work / simulated_seconds;
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
