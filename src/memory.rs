use windows::Win32::System::ProcessStatus::*;
use windows::Win32::Foundation::{HANDLE, CloseHandle};
use windows::Win32::System::Diagnostics::ToolHelp::{PROCESS_MEMORY_COUNTERS, GetProcessMemoryInfo};
use log::{info, error};
use std::{thread, time::Duration};

fn monitor_process_memory(process_name: &str) -> Result<()> {
    loop {
        let mut processes = Vec::new();
        let success = unsafe { EnumProcesses(Some(&mut processes)) };

        if success {
            for &pid in &processes {
                if pid == 0 {
                    continue;
                }

                let process_handle = unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid) };
                if process_handle.is_invalid() {
                    continue;
                }

                let mut exe_name = [0u16; 260];
                let mut size = exe_name.len() as u32;

                let result = unsafe {
                    QueryFullProcessImageNameW(
                        process_handle,
                        0,
                        exe_name.as_mut_ptr(),
                        &mut size,
                    )
                };

                if result != 0 {
                    let exe_path = String::from_utf16_lossy(&exe_name[..size as usize]);
                    if exe_path.to_lowercase().contains(&process_name.to_lowercase()) {
                        let mut pmc = PROCESS_MEMORY_COUNTERS::default();
                        let mem_result = unsafe { GetProcessMemoryInfo(process_handle, &mut pmc, std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32) };
                        if mem_result.as_bool() {
                            let used_memory = pmc.WorkingSetSize as f64 / (1024.0 * 1024.0);
                            info!("Process {} is using {:.2} MB of memory", process_name, used_memory);
                            // Implement threshold alerts if necessary
                        }
                    }
                }

                unsafe { CloseHandle(process_handle) };
            }
        } else {
            error!("Failed to enumerate processes");
        }

        thread::sleep(Duration::from_secs(10));
    }
}
