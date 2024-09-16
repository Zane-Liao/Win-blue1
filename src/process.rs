use windows::Win32::System::ProcessStatus::*;
use windows::Win32::Foundation::{HANDLE, CloseHandle};
use log::{info, error};
use std::{thread, time::Duration};

fn monitor_process(process_name: &str) -> Result<()> {
    loop {
        let mut found = false;
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
                        found = true;
                        break;
                    }
                }

                unsafe { CloseHandle(process_handle) };
            }

            if !found {
                let msg = format!("Process {} has crashed or is not running", process_name);
                info!("{}", msg);
                log_crash(&msg);
                // Optionally, send a notification or take other actions
            }
        } else {
            error!("Failed to enumerate processes");
        }

        thread::sleep(Duration::from_secs(5));
    }
}
