use clap::{Arg, Command};
use std::thread;
use std::time::Duration;
use log::{info, error};
use std::process;

#[tokio::main]
async fn main() {
    // Define CLI arguments using Clap
    let matches = Command::new("Crash Monitor CLI")
        .version("1.0")
        .author("Your Name <your_email@example.com>")
        .about("Real-time monitoring of Windows system and application crashes")
        .arg(
            Arg::new("process")
                .short('p')
                .long("process")
                .value_name("PROCESS_NAME")
                .help("Specify the name of the process to monitor")
                .takes_value(true),
        )
        .arg(
            Arg::new("memory")
                .short('m')
                .long("memory")
                .help("Enable memory monitoring"),
        )
        .arg(
            Arg::new("kernel")
                .short('k')
                .long("kernel")
                .help("Enable kernel-level monitoring"),
        )
        .get_matches();

    let process_name = matches.value_of("process");
    let enable_memory = matches.is_present("memory");
    let enable_kernel = matches.is_present("kernel");

    // Initialize logger
    env_logger::init();

    // Display configuration
    println!("Monitoring Process: {:?}", process_name);
    println!("Memory Monitoring Enabled: {}", enable_memory);
    println!("Kernel Monitoring Enabled: {}", enable_kernel);

    // Start system-level crash monitoring
    if enable_kernel || true { // Assuming system-level monitoring is always enabled
        thread::spawn(|| {
            if let Err(e) = monitor_system_events() {
                error!("System-level monitoring error: {:?}", e);
            }
        });
    }

    // Start application-level crash monitoring
    if let Some(proc) = process_name {
        let proc = proc.to_string();
        thread::spawn(move || {
            if let Err(e) = monitor_process(&proc) {
                error!("Application-level monitoring error: {:?}", e);
            }
        });
    }

    // Start memory monitoring
    if enable_memory {
        thread::spawn(|| {
            if let Err(e) = monitor_memory() {
                error!("Memory monitoring error: {:?}", e);
            }
        });

        if let Some(proc) = process_name {
            let proc = proc.to_string();
            thread::spawn(move || {
                if let Err(e) = monitor_process_memory(&proc) {
                    error!("Process memory monitoring error: {:?}", e);
                }
            });
        }
    }

    // Start kernel-level monitoring
    if enable_kernel {
        thread::spawn(|| {
            if let Err(e) = monitor_kernel_events() {
                error!("Kernel-level monitoring error: {:?}", e);
            }
        });
    }

    // Keep the main thread alive
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
