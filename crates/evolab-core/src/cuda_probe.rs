use serde::{Deserialize, Serialize};

use crate::{CudaDeviceInfo, DevicePerformance, ProbeSpec, SimulationConfig, schedule_gpu_work};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CudaProbeBatchReport {
    pub backend: String,
    pub worlds_evaluated: usize,
    pub steps_per_world: usize,
    pub simulated_seconds_per_world: f32,
    pub final_position: [f32; 3],
    pub final_linear_velocity: [f32; 3],
    pub wall_seconds: f64,
    pub worlds_per_second: f64,
    pub physics_steps_per_second: f64,
    pub devices: Vec<DevicePerformance>,
}

pub fn discover_cuda_devices() -> Result<Vec<CudaDeviceInfo>, String> {
    platform::discover_devices()
}

pub fn run_cuda_probe_batch(
    config: &SimulationConfig,
    probe: &ProbeSpec,
    count: usize,
    gpu_ids: &[u32],
) -> Result<CudaProbeBatchReport, String> {
    if count == 0 {
        return Err("batch size must be greater than 0".into());
    }
    validate_cuda_probe_world(config)?;

    let devices = discover_cuda_devices()?;
    if devices.is_empty() {
        return Err("no CUDA devices were found".into());
    }

    let selected = if gpu_ids.is_empty() {
        devices.iter().map(|device| device.id).collect::<Vec<_>>()
    } else {
        for id in gpu_ids {
            if !devices.iter().any(|device| device.id == *id) {
                return Err(format!("CUDA device {id} is not available"));
            }
        }
        gpu_ids.to_vec()
    };

    let assignments = schedule_gpu_work(count, &selected)?;
    platform::run_batch(config, probe, count, &assignments)
}

fn validate_cuda_probe_world(config: &SimulationConfig) -> Result<(), String> {
    use crate::TerrainKind;

    config.validate()?;

    if config.world.terrain != TerrainKind::Flat {
        return Err("CUDA probe fast path currently supports flat terrain only".into());
    }
    if config.world.walls_enabled
        || config.world.blocks_enabled
        || config.world.gaps_enabled
        || config.world.pits_enabled
    {
        return Err("CUDA probe fast path currently supports worlds without obstacles".into());
    }
    if config.world.gravity[0].abs() > 1.0e-6 || config.world.gravity[2].abs() > 1.0e-6 {
        return Err("CUDA probe fast path currently supports vertical gravity only".into());
    }
    Ok(())
}

#[cfg(not(windows))]
mod platform {
    use crate::{CudaDeviceInfo, DeviceWorkAssignment, ProbeSpec, SimulationConfig};

    use super::CudaProbeBatchReport;

    pub fn discover_devices() -> Result<Vec<CudaDeviceInfo>, String> {
        Ok(Vec::new())
    }

    pub fn run_batch(
        _config: &SimulationConfig,
        _probe: &ProbeSpec,
        _count: usize,
        _assignments: &[DeviceWorkAssignment],
    ) -> Result<CudaProbeBatchReport, String> {
        Err("the built-in CUDA probe backend currently supports Windows only".into())
    }
}

#[cfg(windows)]
mod platform {
    use std::{
        ffi::{CString, c_char, c_void},
        mem::transmute,
        ptr::null_mut,
        thread,
        time::Instant,
    };

    use crate::{
        CudaDeviceInfo, DevicePerformance, DeviceWorkAssignment, ProbeSpec, SimulationConfig,
    };

    use super::CudaProbeBatchReport;

    type CuResult = i32;
    type CuDevice = i32;
    type CuContext = *mut c_void;
    type CuModule = *mut c_void;
    type CuFunction = *mut c_void;
    type CuStream = *mut c_void;
    type CuDevicePtr = u64;

    const CUDA_SUCCESS: CuResult = 0;

    type CuInit = unsafe extern "system" fn(u32) -> CuResult;
    type CuDeviceGetCount = unsafe extern "system" fn(*mut i32) -> CuResult;
    type CuDeviceGet = unsafe extern "system" fn(*mut CuDevice, i32) -> CuResult;
    type CuDeviceGetName = unsafe extern "system" fn(*mut c_char, i32, CuDevice) -> CuResult;
    type CuDeviceTotalMem = unsafe extern "system" fn(*mut usize, CuDevice) -> CuResult;
    type CuDeviceGetAttribute = unsafe extern "system" fn(*mut i32, i32, CuDevice) -> CuResult;
    type CuCtxCreate = unsafe extern "system" fn(*mut CuContext, u32, CuDevice) -> CuResult;
    type CuCtxDestroy = unsafe extern "system" fn(CuContext) -> CuResult;
    type CuCtxSynchronize = unsafe extern "system" fn() -> CuResult;
    type CuMemGetInfo = unsafe extern "system" fn(*mut usize, *mut usize) -> CuResult;
    type CuMemAlloc = unsafe extern "system" fn(*mut CuDevicePtr, usize) -> CuResult;
    type CuMemFree = unsafe extern "system" fn(CuDevicePtr) -> CuResult;
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

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LoadLibraryA(name: *const c_char) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const c_char) -> *mut c_void;
        fn FreeLibrary(module: *mut c_void) -> i32;
    }

    struct CudaApi {
        library: *mut c_void,
        init: CuInit,
        device_get_count: CuDeviceGetCount,
        device_get: CuDeviceGet,
        device_get_name: CuDeviceGetName,
        device_total_mem: CuDeviceTotalMem,
        device_get_attribute: CuDeviceGetAttribute,
        ctx_create: CuCtxCreate,
        ctx_destroy: CuCtxDestroy,
        ctx_synchronize: CuCtxSynchronize,
        mem_get_info: CuMemGetInfo,
        mem_alloc: CuMemAlloc,
        mem_free: CuMemFree,
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
            let name = CString::new("nvcuda.dll").expect("static string");
            let library = unsafe { LoadLibraryA(name.as_ptr()) };
            if library.is_null() {
                return Err(
                    "CUDA driver library nvcuda.dll was not found; install/update the NVIDIA driver"
                        .into(),
                );
            }

            unsafe fn symbol(
                library: *mut c_void,
                name: &'static [u8],
            ) -> Result<*mut c_void, String> {
                let pointer = unsafe { GetProcAddress(library, name.as_ptr().cast::<c_char>()) };
                if pointer.is_null() {
                    let display = String::from_utf8_lossy(&name[..name.len() - 1]);
                    Err(format!("CUDA driver symbol {display} is unavailable"))
                } else {
                    Ok(pointer)
                }
            }

            macro_rules! load {
                ($name:literal, $ty:ty) => {{
                    let pointer = unsafe { symbol(library, concat!($name, "\0").as_bytes())? };
                    unsafe { transmute::<*mut c_void, $ty>(pointer) }
                }};
            }

            let api = Self {
                library,
                init: load!("cuInit", CuInit),
                device_get_count: load!("cuDeviceGetCount", CuDeviceGetCount),
                device_get: load!("cuDeviceGet", CuDeviceGet),
                device_get_name: load!("cuDeviceGetName", CuDeviceGetName),
                device_total_mem: load!("cuDeviceTotalMem_v2", CuDeviceTotalMem),
                device_get_attribute: load!("cuDeviceGetAttribute", CuDeviceGetAttribute),
                ctx_create: load!("cuCtxCreate_v2", CuCtxCreate),
                ctx_destroy: load!("cuCtxDestroy_v2", CuCtxDestroy),
                ctx_synchronize: load!("cuCtxSynchronize", CuCtxSynchronize),
                mem_get_info: load!("cuMemGetInfo_v2", CuMemGetInfo),
                mem_alloc: load!("cuMemAlloc_v2", CuMemAlloc),
                mem_free: load!("cuMemFree_v2", CuMemFree),
                memcpy_dtoh: load!("cuMemcpyDtoH_v2", CuMemcpyDtoH),
                module_load_data: load!("cuModuleLoadData", CuModuleLoadData),
                module_unload: load!("cuModuleUnload", CuModuleUnload),
                module_get_function: load!("cuModuleGetFunction", CuModuleGetFunction),
                launch_kernel: load!("cuLaunchKernel", CuLaunchKernel),
            };

            check(unsafe { (api.init)(0) }, "cuInit")?;
            Ok(api)
        }
    }

    fn check(result: CuResult, operation: &str) -> Result<(), String> {
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

    struct DeviceMemory<'a> {
        api: &'a CudaApi,
        pointer: CuDevicePtr,
    }

    impl Drop for DeviceMemory<'_> {
        fn drop(&mut self) {
            if self.pointer != 0 {
                unsafe {
                    let _ = (self.api.mem_free)(self.pointer);
                }
            }
        }
    }

    pub fn discover_devices() -> Result<Vec<CudaDeviceInfo>, String> {
        let api = CudaApi::load()?;
        let mut count = 0_i32;
        check(
            unsafe { (api.device_get_count)(&mut count) },
            "cuDeviceGetCount",
        )?;

        let mut devices = Vec::with_capacity(count.max(0) as usize);
        for ordinal in 0..count {
            let mut device = 0;
            check(
                unsafe { (api.device_get)(&mut device, ordinal) },
                "cuDeviceGet",
            )?;

            let mut name_buffer = [0_i8; 256];
            check(
                unsafe {
                    (api.device_get_name)(
                        name_buffer.as_mut_ptr(),
                        name_buffer.len() as i32,
                        device,
                    )
                },
                "cuDeviceGetName",
            )?;
            let name_len = name_buffer
                .iter()
                .position(|value| *value == 0)
                .unwrap_or(name_buffer.len());
            let name_bytes = name_buffer[..name_len]
                .iter()
                .map(|value| *value as u8)
                .collect::<Vec<_>>();
            let name = String::from_utf8_lossy(&name_bytes).into_owned();

            let mut total_memory = 0usize;
            check(
                unsafe { (api.device_total_mem)(&mut total_memory, device) },
                "cuDeviceTotalMem",
            )?;

            let mut major = 0_i32;
            let mut minor = 0_i32;
            let mut multiprocessor_count = 0_i32;
            // CU_DEVICE_ATTRIBUTE_MULTIPROCESSOR_COUNT and compute capability.
            check(
                unsafe { (api.device_get_attribute)(&mut multiprocessor_count, 16, device) },
                "cuDeviceGetAttribute(multiprocessor_count)",
            )?;
            check(
                unsafe { (api.device_get_attribute)(&mut major, 75, device) },
                "cuDeviceGetAttribute(major)",
            )?;
            check(
                unsafe { (api.device_get_attribute)(&mut minor, 76, device) },
                "cuDeviceGetAttribute(minor)",
            )?;

            let mut context = null_mut();
            check(
                unsafe { (api.ctx_create)(&mut context, 0, device) },
                "cuCtxCreate",
            )?;
            let context_guard = ContextGuard { api: &api, context };
            let mut free_memory = 0usize;
            let mut context_total = 0usize;
            check(
                unsafe { (api.mem_get_info)(&mut free_memory, &mut context_total) },
                "cuMemGetInfo",
            )?;
            drop(context_guard);

            devices.push(CudaDeviceInfo {
                id: ordinal as u32,
                name,
                total_memory_bytes: total_memory as u64,
                free_memory_bytes: free_memory as u64,
                compute_capability_major: major,
                compute_capability_minor: minor,
                multiprocessor_count,
            });
        }

        Ok(devices)
    }

    pub fn run_batch(
        config: &SimulationConfig,
        probe: &ProbeSpec,
        count: usize,
        assignments: &[DeviceWorkAssignment],
    ) -> Result<CudaProbeBatchReport, String> {
        let started = Instant::now();
        let mut handles = Vec::with_capacity(assignments.len());

        for assignment in assignments {
            let config = config.clone();
            let probe = *probe;
            let assignment = assignment.clone();
            handles.push(thread::spawn(move || {
                run_device_batch(&config, &probe, &assignment)
            }));
        }

        let mut device_stats = Vec::with_capacity(handles.len());
        let mut first_position = None;
        let mut first_velocity = None;

        for handle in handles {
            let result = handle
                .join()
                .map_err(|_| "CUDA worker thread panicked".to_string())??;
            if first_position.is_none() {
                first_position = Some(result.final_position);
                first_velocity = Some(result.final_velocity);
            }
            device_stats.push(result.performance);
        }

        let wall_seconds = started.elapsed().as_secs_f64().max(f64::EPSILON);
        let steps = config.step_count();
        let physics_steps = count as u64 * steps as u64;

        Ok(CudaProbeBatchReport {
            backend: if assignments.len() > 1 {
                "cuda-probe-multi-gpu".into()
            } else {
                "cuda-probe".into()
            },
            worlds_evaluated: count,
            steps_per_world: steps,
            simulated_seconds_per_world: steps as f32 * config.dt,
            final_position: first_position.unwrap_or([0.0; 3]),
            final_linear_velocity: first_velocity.unwrap_or([0.0; 3]),
            wall_seconds,
            worlds_per_second: count as f64 / wall_seconds,
            physics_steps_per_second: physics_steps as f64 / wall_seconds,
            devices: device_stats,
        })
    }

    struct DeviceBatchResult {
        final_position: [f32; 3],
        final_velocity: [f32; 3],
        performance: DevicePerformance,
    }

    fn run_device_batch(
        config: &SimulationConfig,
        probe: &ProbeSpec,
        assignment: &DeviceWorkAssignment,
    ) -> Result<DeviceBatchResult, String> {
        let api = CudaApi::load()?;

        let mut device = 0;
        check(
            unsafe { (api.device_get)(&mut device, assignment.device_id as i32) },
            "cuDeviceGet",
        )?;

        let mut context = null_mut();
        check(
            unsafe { (api.ctx_create)(&mut context, 0, device) },
            "cuCtxCreate",
        )?;
        let _context_guard = ContextGuard { api: &api, context };

        let ptx = CString::new(PROBE_PTX).expect("embedded PTX contains no NUL");
        let mut module = null_mut();
        check(
            unsafe { (api.module_load_data)(&mut module, ptx.as_ptr().cast()) },
            "cuModuleLoadData",
        )?;
        let _module_guard = ModuleGuard { api: &api, module };

        let kernel_name = CString::new("simulate_probes").expect("static kernel name");
        let mut function = null_mut();
        check(
            unsafe { (api.module_get_function)(&mut function, module, kernel_name.as_ptr()) },
            "cuModuleGetFunction",
        )?;

        let byte_len = assignment.count * size_of::<f32>();
        let mut out_y = 0_u64;
        let mut out_vy = 0_u64;
        check(
            unsafe { (api.mem_alloc)(&mut out_y, byte_len) },
            "cuMemAlloc(out_y)",
        )?;
        let _out_y_guard = DeviceMemory {
            api: &api,
            pointer: out_y,
        };
        check(
            unsafe { (api.mem_alloc)(&mut out_vy, byte_len) },
            "cuMemAlloc(out_vy)",
        )?;
        let _out_vy_guard = DeviceMemory {
            api: &api,
            pointer: out_vy,
        };

        let mut out_y_arg = out_y;
        let mut out_vy_arg = out_vy;
        let mut count_arg = assignment.count as u32;
        let mut initial_y = probe.initial_position[1];
        let mut half_y = probe.half_extents[1];
        let mut gravity_y = config.world.gravity[1];
        let mut dt = config.dt;
        let mut steps = config.step_count() as u32;
        let mut ground_y = config
            .world
            .surface_height_at(probe.initial_position[0], probe.initial_position[2])
            .unwrap_or(0.0);
        let mut restitution = probe.restitution;

        let mut params = [
            (&mut out_y_arg as *mut CuDevicePtr).cast::<c_void>(),
            (&mut out_vy_arg as *mut CuDevicePtr).cast::<c_void>(),
            (&mut count_arg as *mut u32).cast::<c_void>(),
            (&mut initial_y as *mut f32).cast::<c_void>(),
            (&mut half_y as *mut f32).cast::<c_void>(),
            (&mut gravity_y as *mut f32).cast::<c_void>(),
            (&mut dt as *mut f32).cast::<c_void>(),
            (&mut steps as *mut u32).cast::<c_void>(),
            (&mut ground_y as *mut f32).cast::<c_void>(),
            (&mut restitution as *mut f32).cast::<c_void>(),
        ];

        let block_size = 256_u32;
        let grid_size = (assignment.count as u32).div_ceil(block_size);
        let started = Instant::now();

        check(
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
            "cuLaunchKernel",
        )?;
        check(unsafe { (api.ctx_synchronize)() }, "cuCtxSynchronize")?;

        let wall_seconds = started.elapsed().as_secs_f64().max(f64::EPSILON);

        let mut host_y = vec![0.0_f32; assignment.count];
        let mut host_vy = vec![0.0_f32; assignment.count];
        check(
            unsafe { (api.memcpy_dtoh)(host_y.as_mut_ptr().cast::<c_void>(), out_y, byte_len) },
            "cuMemcpyDtoH(out_y)",
        )?;
        check(
            unsafe { (api.memcpy_dtoh)(host_vy.as_mut_ptr().cast::<c_void>(), out_vy, byte_len) },
            "cuMemcpyDtoH(out_vy)",
        )?;

        let steps_per_world = config.step_count();
        let physics_steps = assignment.count as u64 * steps_per_world as u64;
        let performance = DevicePerformance {
            device_id: assignment.device_id,
            items: assignment.count,
            wall_seconds,
            items_per_second: assignment.count as f64 / wall_seconds,
            physics_steps_per_second: physics_steps as f64 / wall_seconds,
            cuda: Default::default(),
        };

        Ok(DeviceBatchResult {
            final_position: [
                probe.initial_position[0],
                host_y[0],
                probe.initial_position[2],
            ],
            final_velocity: [0.0, host_vy[0], 0.0],
            performance,
        })
    }

    const PROBE_PTX: &str = r#"
.version 6.0
.target sm_50
.address_size 64

.visible .entry simulate_probes(
    .param .u64 p_out_y,
    .param .u64 p_out_vy,
    .param .u32 p_count,
    .param .f32 p_initial_y,
    .param .f32 p_half_y,
    .param .f32 p_gravity_y,
    .param .f32 p_dt,
    .param .u32 p_steps,
    .param .f32 p_ground_y,
    .param .f32 p_restitution
)
{
    .reg .pred %p<4>;
    .reg .b32 %r<10>;
    .reg .b64 %rd<8>;
    .reg .f32 %f<12>;

    ld.param.u64 %rd1, [p_out_y];
    ld.param.u64 %rd2, [p_out_vy];
    ld.param.u32 %r5, [p_count];

    mov.u32 %r1, %ctaid.x;
    mov.u32 %r2, %ntid.x;
    mov.u32 %r3, %tid.x;
    mad.lo.s32 %r4, %r1, %r2, %r3;

    setp.ge.u32 %p1, %r4, %r5;
    @%p1 ret;

    mul.wide.u32 %rd3, %r4, 4;
    add.s64 %rd4, %rd1, %rd3;
    add.s64 %rd5, %rd2, %rd3;

    ld.param.f32 %f1, [p_initial_y];
    ld.param.f32 %f2, [p_half_y];
    ld.param.f32 %f3, [p_gravity_y];
    ld.param.f32 %f4, [p_dt];
    ld.param.u32 %r6, [p_steps];
    ld.param.f32 %f5, [p_ground_y];
    ld.param.f32 %f6, [p_restitution];

    mov.f32 %f7, %f1;
    mov.f32 %f8, 0f00000000;
    mov.u32 %r7, 0;

LOOP:
    setp.ge.u32 %p2, %r7, %r6;
    @%p2 bra DONE;

    fma.rn.f32 %f8, %f3, %f4, %f8;
    fma.rn.f32 %f7, %f8, %f4, %f7;
    add.f32 %f9, %f5, %f2;

    setp.ge.f32 %p3, %f7, %f9;
    @%p3 bra NO_COLLIDE;

    mov.f32 %f7, %f9;
    setp.ge.f32 %p3, %f8, 0f00000000;
    @%p3 bra NO_COLLIDE;
    neg.f32 %f10, %f8;
    mul.f32 %f8, %f10, %f6;

NO_COLLIDE:
    add.u32 %r7, %r7, 1;
    bra LOOP;

DONE:
    st.global.f32 [%rd4], %f7;
    st.global.f32 [%rd5], %f8;
    ret;
}
"#;
}
