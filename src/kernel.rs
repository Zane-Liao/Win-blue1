fn monitor_kernel_events() -> Result<()> {
    unsafe {
        // Open the System event log
        let h_event_log = OpenEventLogW(None, wide_string("System").as_ptr());

        if h_event_log == HANDLE::default() {
            error!("Failed to open system event log");
            return Ok(());
        }

        // Read the event log
        let mut buffer: [u8; 65536] = [0; 65536];
        let mut bytes_read          = 0;
        let mut min_bytes_needed    = 0;

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
                // Filter kernel-related events, e.g., Event ID 6008 (unexpected shutdown)
                if (record.event_id == 6008 || record.source.contains("Kernel")) {
                    info!("Detected kernel-related event: {:?}", record);
                    log_crash(&format!("Kernel-related event: {:?}", record));
                }
            }
        }

        CloseEventLog(h_event_log);
    }

    Ok(())
}
