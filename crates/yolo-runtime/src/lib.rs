use serde::{Deserialize, Serialize};
use std::ffi::c_char;
use sysinfo::System;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformTopology {
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub hostname: String,
    pub cpu_arch: String,
    pub cpu_count: usize,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub avx2_supported: bool,
    pub avx512_supported: bool,
}

#[unsafe(no_mangle)]
pub extern "C" fn yolo_platform_get_topology() -> *mut c_char {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpuid = raw_cpuid::CpuId::new();
    let avx2 = cpuid
        .get_extended_feature_info()
        .map(|f| f.has_avx2())
        .unwrap_or(false);
    let avx512 = cpuid
        .get_extended_feature_info()
        .map(|f| f.has_avx512f())
        .unwrap_or(false);

    let topology = PlatformTopology {
        os_name: System::name().unwrap_or_else(|| "Unknown".into()),
        os_version: System::os_version().unwrap_or_else(|| "Unknown".into()),
        kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".into()),
        hostname: System::host_name().unwrap_or_else(|| "localhost".into()),
        cpu_arch: std::env::consts::ARCH.to_string(),
        cpu_count: sys.cpus().len(),
        total_memory_mb: sys.total_memory() / (1024 * 1024),
        available_memory_mb: sys.available_memory() / (1024 * 1024),
        avx2_supported: avx2,
        avx512_supported: avx512,
    };

    let json = serde_json::to_string(&topology).unwrap_or_else(|_| "{}".into());
    std::ffi::CString::new(json).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn yolo_platform_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = std::ffi::CString::from_raw(ptr);
        }
    }
}
