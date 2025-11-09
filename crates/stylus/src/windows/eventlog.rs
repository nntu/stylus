use log::{Level};

const EVENT_SOURCE_NAME: &str = "StylusMonitor";

pub fn register_event_source() -> Result<(), Box<dyn std::error::Error>> {
    // For now, just return success
    // In a full implementation, you would register with Windows Event Log
    Ok(())
}

pub fn write_event_log(level: Level, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    // For now, just print to console
    // In a full implementation, you would write to Windows Event Log
    match level {
        Level::Error => eprintln!("[ERROR] {}", message),
        Level::Warn => eprintln!("[WARN] {}", message),
        Level::Info => println!("[INFO] {}", message),
        Level::Debug => println!("[DEBUG] {}", message),
        Level::Trace => println!("[TRACE] {}", message),
    }
    Ok(())
}

// A simple logger that writes to Windows Event Log (console for now)
pub struct EventLogger;

impl log::Log for EventLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            if let Err(e) = write_event_log(record.level(), &record.args().to_string()) {
                eprintln!("Failed to write to event log: {}", e);
            }
        }
    }

    fn flush(&self) {
        // Windows Event Log doesn't need flushing
    }
}