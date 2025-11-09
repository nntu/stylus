use std::ffi::OsString;
use std::thread;
use windows_service::{
    service::{
        ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceType,
    },
    service_manager::{ServiceManager, ServiceManagerAccess},
    Result,
};

const SERVICE_NAME: &str = "StylusMonitor";
const SERVICE_DISPLAY_NAME: &str = "Stylus Infrastructure Monitor";
const SERVICE_DESCRIPTION: &str = "Professional infrastructure monitoring service with web interface";

pub fn install_service(executable_path: &str, config_path: Option<&str>) -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_binary_path = std::env::current_exe()
        .map_err(|e| windows_service::Error::Winapi(std::io::Error::from(e)))?;

    // Create service arguments
    let mut service_args: Vec<OsString> = vec![
        "run".into(),
        config_path.unwrap_or("").into(),
    ];

    // Add service mode flag
    service_args.push("--service".into());

    let service_info = ServiceInfo {
        name: OsString::from(SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: windows_service::service::ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: service_binary_path,
        launch_arguments: service_args,
        dependencies: vec![],
        account_name: None, // Run as LocalSystem
        account_password: None,
    };

    let _service = service_manager.create_service(&service_info, ServiceAccess::empty())?;

    println!("Service '{}' installed successfully", SERVICE_NAME);
    println!("Service name: {}", SERVICE_NAME);
    println!("Display name: {}", SERVICE_DISPLAY_NAME);
    println!("Description: {}", SERVICE_DESCRIPTION);

    Ok(())
}

pub fn uninstall_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::DELETE | ServiceAccess::STOP;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    // Stop the service if it's running
    let service_status = service.query_status()?;
    if service_status.current_state != windows_service::service::ServiceState::Stopped {
        println!("Stopping service...");
        service.stop()?;

        // Wait for service to stop
        thread::sleep(std::time::Duration::from_secs(5));
    }

    // Delete the service
    service.delete()?;

    println!("Service '{}' uninstalled successfully", SERVICE_NAME);

    Ok(())
}

pub fn start_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::START;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    service.start(&[] as &[&str])?;

    println!("Service '{}' started successfully", SERVICE_NAME);

    Ok(())
}

pub fn stop_service() -> Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::STOP;
    let service = service_manager.open_service(SERVICE_NAME, service_access)?;

    service.stop()?;

    println!("Service '{}' stopped successfully", SERVICE_NAME);

    Ok(())
}