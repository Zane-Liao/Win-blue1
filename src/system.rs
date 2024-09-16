use windows::{
    core::*,
    Win32::System::EventLog::*,
    Win32::Foundation::HANDLE,
};
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::SystemTime;
use log::{info, error};

#[derive(Debug, Serialize)]
struct EventRecord {
    event_id: u32,
    source: String,
    message: String,
}

#[derive(Serialize)]
struct CrashLog {
    timestamp: u128,
    event: String,
}

fn log_crash(event: &str) {
    let log = CrashLog {
        timestamp: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis(),
        event: event.to_string(),
    };

    let log_json = serde_json::to_string(&log).unwrap();

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("crash_logs.json")
        .unwrap();

    writeln!(file, "{}", log_json).unwrap();
}

unsafe fn parse_event_log(buffer: &[u8]) -> Vec<EventRecord> {
    let mut records = Vec::new();
    let mut offset = 0;

    while offset < buffer.len() {
        if buffer.len() - offset < std::mem::size_of::<EVENTLOGRECORD>() {
            break;
        }

        let record_ptr = buffer.as_ptr().add(offset) as *const EVENTLOGRECORD;
        let record = &*record_ptr;

        let event_id = record.EventID & 0xFFFF;
        let source_ptr = buffer.as_ptr().add(record.StringOffset as usize) as *const u16;
        let source = wide_to_string(source_ptr);

        let message_ptr = buffer.as_ptr().add(record.StringOffset as usize + (record.NumStrings as u32 * 2) as usize) as *const u16;
        let message = wide_to_string(message_ptr);

        records.push(EventRecord {
            event_id,
            source,
            message,
        });

        offset += record.Length as usize;
    }

    records
}

fn wide_to_string(ptr: *const u16) -> String {
    let mut len = 0;
    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
    }
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf16_lossy(slice)
}

fn wide_string(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

fn monitor_system_events() -> Result<()> {
    unsafe {
        // Open the System event log
        let h_event_log = OpenEventLogW(None, wide_string("System").as_ptr());

        if h_event_log == HANDLE::default() {
            error!("Failed to open system event log");
            return Ok(());
        }

        // Read the event log
        let mut buffer: [u8; 65536] = [0; 65536];
        let mut bytes_read = 0;
        let mut min_bytes_needed = 0;

        loop {
            let result = ReadEventLogW(
                h_event_log,
                EVENTLOG_SEQUENTIAL_READ | EVENTLOG_FORWARDS_READ,
                0,
                buffer.as_mut_ptr() as *mut _,
                buffer.len() as u32,
                &mut bytes_read,
                &mut min_bytes_needed,
            );

            if result == 0 {
                // No more events
                break;
            }

            // Parse EVENTLOG_RECORD
            let records = parse_event_log(&buffer[..bytes_read as usize]);

            for record in records {
                // Filter BSOD events (Event ID 1001 from source "BugCheck")
                if record.event_id == 1001 && record.source == "BugCheck" {
                    info!("Detected BSOD event: {:?}", record);
                    log_crash(&format!("BSOD event: {:?}", record));
                }
            }
        }

        CloseEventLog(h_event_log);
    }

    Ok(())
}
