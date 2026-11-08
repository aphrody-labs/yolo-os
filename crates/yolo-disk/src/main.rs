use std::env;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::null_mut;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Ioctl::{
    DEVICE_DATA_SET_RANGE, DEVICE_DSM_FLAG_TRIM_NOT_EXCEED_MAX_EXTENT, DEVICE_MANAGE_DATA_SET_ATTRIBUTES,
    DeviceDsmAction_Trim, IOCTL_STORAGE_MANAGE_DATA_SET_ATTRIBUTES,
};

fn send_volume_trim(volume_letter: char) -> Result<(), String> {
    let path = format!(r"\\.\{}:", volume_letter);
    let mut wide: Vec<u16> = path.encode_utf16().collect();
    wide.push(0);

    let handle: HANDLE = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            FILE_GENERIC_READ.0 | FILE_GENERIC_WRITE.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            HANDLE(null_mut()),
        )
    }.map_err(|e| format!("CreateFileW failed for {}: {}", path, e))?;

    if handle == INVALID_HANDLE_VALUE {
        return Err(format!("Invalid handle for {}", path));
    }

    struct DsmInput {
        dsm: DEVICE_MANAGE_DATA_SET_ATTRIBUTES,
        range: DEVICE_DATA_SET_RANGE,
    }

    let mut input = DsmInput {
        dsm: DEVICE_MANAGE_DATA_SET_ATTRIBUTES {
            Size: size_of::<DEVICE_MANAGE_DATA_SET_ATTRIBUTES>() as u32,
            Action: DeviceDsmAction_Trim,
            Flags: DEVICE_DSM_FLAG_TRIM_NOT_EXCEED_MAX_EXTENT,
            ParameterBlockOffset: 0,
            ParameterBlockLength: 0,
            DataSetRangesOffset: size_of::<DEVICE_MANAGE_DATA_SET_ATTRIBUTES>() as u32,
            DataSetRangesLength: size_of::<DEVICE_DATA_SET_RANGE>() as u32,
        },
        range: DEVICE_DATA_SET_RANGE {
            StartingOffset: 0,
            LengthInBytes: 0, // 0 = entire volume extent request
        },
    };

    let mut bytes_returned: u32 = 0;
    let res = unsafe {
        windows::Win32::System::IO::DeviceIoControl(
            handle,
            IOCTL_STORAGE_MANAGE_DATA_SET_ATTRIBUTES,
            Some(&mut input as *mut _ as *mut c_void),
            size_of::<DsmInput>() as u32,
            None,
            0,
            Some(&mut bytes_returned),
            None,
        )
    };

    unsafe { let _ = CloseHandle(handle); };

    res.map_err(|e| format!("IOCTL_STORAGE_MANAGE_DATA_SET_ATTRIBUTES Trim error: {}", e))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("[yolo-disk] Win32 Autonomous SSD/NTFS Optimization Engine (Rust 2024)");

    let drive_letter = args.get(1).and_then(|s| s.chars().next()).unwrap_or('C').to_ascii_uppercase();
    println!("[yolo-disk] Target Volume: {}:", drive_letter);

    match send_volume_trim(drive_letter) {
        Ok(_) => println!("[yolo-disk] Win32 NVMe TRIM (DeviceDsmAction_Trim) dispatched successfully."),
        Err(e) => {
            println!("[yolo-disk] Win32 DeviceIoControl TRIM returned: {}", e);
            println!("[yolo-disk] Operating system Volume Optimization / Defrag API can also dispatch TRIM.");
        }
    }
}
