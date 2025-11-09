use std::ffi::OsString;
use std::sync::mpsc;
use std::time::Duration;
use std::{thread, time::Instant};

use log::{error, info, warn};
use tokio::runtime::Runtime;
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher, Result,
};

use crate::config::parse_config_from_args;
use crate::http::run as run_server;

static SERVICE_NAME: &str = "StylusMonitor";
static SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

define_windows_service!(ffi_service_main, service_main);

pub fn run_service() -> Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
}

fn service_main(arguments: Vec<OsString>) {
    if let Err(e) = run_service_impl(arguments) {
        error!("Service failed to start: {}", e);
    }
}

fn run_service_impl(arguments: Vec<OsString>) -> windows_service::Result<()> {
    // Create a channel for receiving service control events
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    // Define service control handler
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop | ServiceControl::Shutdown => {
                info!("Received stop/shutdown signal");
                let _ = shutdown_tx.send(());
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register service control handler
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // Tell the system that the service is starting
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::StartPending,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(10),
        process_id: None,
    })?;

    // Start the service in a separate thread
    let service_thread = thread::spawn(move || {
        // Create Tokio runtime
        let rt = match Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                error!("Failed to create Tokio runtime: {}", e);
                return;
            }
        };

        // Run the main application
        rt.block_on(async {
            // Parse configuration (use default or environment variables for service mode)
            let operation = match parse_config_from_args() {
                Ok(op) => op,
                Err(e) => {
                    error!("Failed to parse configuration: {}", e);
                    return;
                }
            };

            match operation {
                crate::config::OperationMode::Run(config, dry_run) => {
                    info!("Starting Stylus HTTP server");
                    let _ = run_server(config, dry_run).await;
                }
                _ => {
                    warn!("Service mode only supports 'run' operation");
                }
            }
        });
    });

    // Tell the system that the service is running
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    info!("Service started successfully");

    // Wait for shutdown signal
    let _ = shutdown_rx.recv();

    // Tell the system that the service is stopping
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::StopPending,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(5),
        process_id: None,
    })?;

    info!("Service stopping");

    // Wait for the service thread to finish (with timeout)
    let join_handle = service_thread;
    let start_time = Instant::now();
    let timeout = Duration::from_secs(10);

    while start_time.elapsed() < timeout {
        if join_handle.is_finished() {
            let _ = join_handle.join();
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }

    // Tell the system that the service has stopped
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    info!("Service stopped");
    Ok(())
}