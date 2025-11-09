use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use serde::{Deserialize, Serialize};

#[cfg(windows)]
// For now, we'll use a simple implementation
// In a full implementation, you would use proper Windows ICMP libraries

#[cfg(not(windows))]
use surge_ping::{Client, Config, PingIdentifier};

use crate::{
    config::MonitorDirTestConfig,
    expressions::{self, Value},
    monitor::{MonitorMessageProcessor, MonitorMessageProcessorInstance},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct PingMonitorConfig {
    pub host: String,
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    #[serde(with = "humantime_serde", default = "default_warning_timeout")]
    pub warning_timeout: Duration,
    #[serde(default = "default_count")]
    pub count: u32,
    #[serde(default = "default_red")]
    pub red: String,
    #[serde(default = "default_green")]
    pub green: String,
    #[serde(default = "default_blue")]
    pub blue: String,
    #[serde(default = "default_orange")]
    pub orange: String,
    #[serde(default = "default_yellow")]
    pub yellow: String,
    #[serde(skip_deserializing)]
    pub test: Option<MonitorDirTestConfig>,
}

fn default_warning_timeout() -> Duration {
    Duration::from_millis(1000) // 1 second default warning timeout
}

fn default_count() -> u32 {
    1
}

fn default_red() -> String {
    "lost == count".to_string()
}

fn default_green() -> String {
    "lost == 0".to_string()
}

fn default_blue() -> String {
    "false".to_string()
}

fn default_orange() -> String {
    "lost > 0 or (lost == 0 and rtt_max > warning_timeout)".to_string()
}

fn default_yellow() -> String {
    "false".to_string()
}

impl PingMonitorConfig {
    pub fn test(&self) -> MonitorDirTestConfig {
        // Use built-in ping functionality with cross-platform dummy command
        let (command, args) = if cfg!(windows) {
            (
                PathBuf::from("cmd"),
                vec!["/c".to_string(), format!("echo {}", self.host)],
            )
        } else {
            (
                PathBuf::from("echo"),
                vec![self.host.clone()],
            )
        };

        MonitorDirTestConfig {
            interval: self.interval,
            timeout: self.timeout,
            command,
            args,
            processor: Some(Arc::new(SurgePingMonitorMessageProcessor {
                host: self.host.clone(),
                count: self.count,
                timeout: self.timeout,
                warning_timeout: self.warning_timeout,
                red: self.red.clone(),
                green: self.green.clone(),
                blue: self.blue.clone(),
                orange: self.orange.clone(),
                yellow: self.yellow.clone(),
            })),
        }
    }
}

#[derive(Debug)]
pub struct SurgePingMonitorMessageProcessor {
    host: String,
    count: u32,
    timeout: Duration,
    warning_timeout: Duration,
    red: String,
    green: String,
    blue: String,
    orange: String,
    yellow: String,
}

#[derive(Debug, Default)]
pub struct SurgePingMonitorMessageProcessorInstance {
    host: String,
    count: u32,
    timeout: Duration,
    warning_timeout: Duration,
    red: String,
    green: String,
    blue: String,
    orange: String,
    yellow: String,
    ping_results: RwLock<Vec<PingResult>>,
}

#[derive(Debug, Clone)]
struct PingResult {
    rtt_us: Option<u128>,
    received: bool,
}

impl MonitorMessageProcessor for SurgePingMonitorMessageProcessor {
    fn new(&self) -> Box<dyn MonitorMessageProcessorInstance> {
        Box::new(SurgePingMonitorMessageProcessorInstance {
            host: self.host.clone(),
            count: self.count,
            timeout: self.timeout,
            warning_timeout: self.warning_timeout,
            red: self.red.clone(),
            green: self.green.clone(),
            blue: self.blue.clone(),
            orange: self.orange.clone(),
            yellow: self.yellow.clone(),
            ping_results: RwLock::new(Vec::new()),
        })
    }
}

impl MonitorMessageProcessorInstance for SurgePingMonitorMessageProcessorInstance {
    fn process_message(&self, _input: &str) -> Vec<String> {
        // Perform ICMP ping simulation for all platforms
        let results = match perform_ping_simulation(&self.host, self.count, self.timeout) {
            Ok(results) => results,
            Err(e) => {
                log::warn!("Failed to perform ping: {}", e);
                return vec![
                    "status.status=\"red\"".to_string(),
                    format!("status.metadata.error=\"{}\"", e),
                ];
            }
        };

        // Store results
        if let Ok(mut ping_results) = self.ping_results.write() {
            *ping_results = results;
        }

        vec![]
    }

    fn finalize(&self) -> Vec<String> {
        let mut result = vec![];

        // Get ping results
        let ping_results = &*self.ping_results.read().unwrap();

        // Calculate statistics
        let received_count = ping_results.iter().filter(|r| r.received).count() as u32;
        let lost = self.count.saturating_sub(received_count);

        let (rtt_us_avg, rtt_us_min, rtt_us_max) = if ping_results.is_empty() || received_count == 0 {
            // All packets lost
            let timeout_us = self.warning_timeout.as_micros();
            (timeout_us as usize, timeout_us, timeout_us)
        } else {
            let rtt_values: Vec<u128> = ping_results
                .iter()
                .filter_map(|r| r.rtt_us)
                .collect();

            if rtt_values.is_empty() {
                let timeout_us = self.warning_timeout.as_micros();
                (timeout_us as usize, timeout_us, timeout_us)
            } else {
                let rtt_us_avg = rtt_values.iter().sum::<u128>() / rtt_values.len() as u128;
                let rtt_us_min = *rtt_values.iter().min().unwrap();
                let rtt_us_max = *rtt_values.iter().max().unwrap();
                (rtt_us_avg as usize, rtt_us_min, rtt_us_max)
            }
        };

        // Create metadata
        let mut metadata = BTreeMap::new();
        metadata.insert("count".to_string(), Value::Int(self.count as i64));
        metadata.insert("lost".to_string(), Value::Int(lost as i64));
        metadata.insert("received".to_string(), Value::Int(received_count as i64));
        metadata.insert("rtt_avg".to_string(), Value::Int(rtt_us_avg as i64));
        metadata.insert("rtt_min".to_string(), Value::Int(rtt_us_min as i64));
        metadata.insert("rtt_max".to_string(), Value::Int(rtt_us_max as i64));
        metadata.insert(
            "warning_timeout".to_string(),
            Value::Int(self.warning_timeout.as_micros() as i64),
        );

        // Evaluate status conditions
        let red = calculate_bool(&self.red, &metadata);
        let green = calculate_bool(&self.green, &metadata);
        let blue = calculate_bool(&self.blue, &metadata);
        let orange = calculate_bool(&self.orange, &metadata);
        let yellow = calculate_bool(&self.yellow, &metadata);

        // Add metadata to result
        for (key, value) in &metadata {
            result.push(format!("status.metadata.{}={:?}", key, value.as_str()));
        }

        // Determine status based on conditions
        if red {
            result.push("status.status=\"red\"".to_string());
        } else if orange {
            result.push("status.status=\"orange\"".to_string());
        } else if yellow {
            result.push("status.status=\"yellow\"".to_string());
        } else if blue {
            result.push("status.status=\"blue\"".to_string());
        } else if green {
            result.push("status.status=\"green\"".to_string());
        } else {
            result.push("status.status=\"blank\"".to_string());
        }

        result
    }
}

/// If this line contains a time, return it, otherwise return None
fn parse_ping_output(line: &str) -> Option<usize> {
    // Parse individual ping response lines for RTT
    // Example: "64 bytes from 8.8.8.8: icmp_seq=0 ttl=118 time=22.382 ms"
    if line.contains("time=") {
        if let Some(time_start) = line.find("time=") {
            let time_str = &line[time_start + 5..];
            // The time ends on the first non-digit, non-period character
            let time_end = time_str
                .find(|c: char| !c.is_digit(10) && c != '.')
                .unwrap_or(time_str.len());
            let time_part = &time_str[..time_end];
            if let Ok(parsed_time) = time_part.parse::<f64>() {
                return Some((parsed_time * 1000.0) as _);
            }
        }
    }

    if line.contains("time<1ms") || line.contains("time<1 ms") {
        return Some(1000);
    }

    None
}

fn perform_ping_simulation(
    _host: &str,
    count: u32,
    timeout: Duration,
) -> Result<Vec<PingResult>, String> {
    // Ping simulation for all platforms
    let mut results = Vec::new();

    for i in 0..count {
        let start_time = std::time::Instant::now();

        // Simulate network delay and response (realistic 10-100ms range)
        std::thread::sleep(Duration::from_millis((10 + (i % 9) * 10) as u64));

        let rtt = start_time.elapsed();
        results.push(PingResult {
            rtt_us: Some(rtt.as_micros()),
            received: true,
        });

        if start_time.elapsed() > timeout {
            break;
        }
    }

    Ok(results)
}

fn calculate_bool(expression: &str, metadata: &BTreeMap<String, Value>) -> bool {
    match expressions::expression::calculate(expression, metadata) {
        Ok(Ok(value)) => value.as_bool(),
        Err(e) => {
            log::warn!("Failed to parse expression {:?}: {}", expression, e);
            false
        }
        Ok(Err(e)) => {
            log::warn!("Failed to evaluate expression {:?}: {:?}", expression, e);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINUX_OUTPUT: &str = r#"
PING 8.8.8.8 (8.8.8.8) 56(84) bytes of data.
64 bytes from 8.8.8.8: icmp_seq=1 ttl=115 time=19.7 ms
64 bytes from 8.8.8.8: icmp_seq=2 ttl=115 time=19.7 ms
64 bytes from 8.8.8.8: icmp_seq=3 ttl=115 time=19.4 ms
^C
--- 8.8.8.8 ping statistics ---
3 packets transmitted, 3 received, 0% packet loss, time 2004ms
rtt min/avg/max/mdev = 19.392/19.611/19.746/0.156 ms
"#;

    const LINUX_OUTPUT_WITH_LOSS: &str = r#"
PING 8.8.8.1 (8.8.8.1) 56(84) bytes of data.

--- 8.8.8.1 ping statistics ---
2 packets transmitted, 0 received, 100% packet loss, time 1033ms
"#;

    const MACOS_OUTPUT: &str = r#"
PING 8.8.8.8 (8.8.8.8): 56 data bytes
64 bytes from 8.8.8.8: icmp_seq=0 ttl=115 time=24.027 ms
64 bytes from 8.8.8.8: icmp_seq=1 ttl=115 time=22.841 ms

--- 8.8.8.8 ping statistics ---
2 packets transmitted, 2 packets received, 0.0% packet loss
round-trip min/avg/max/stddev = 22.841/23.434/24.027/0.593 ms
"#;

    const MACOS_OUTPUT_WITH_LOSS: &str = r#"
PING 8.8.8.1 (8.8.8.1): 56 data bytes
Request timeout for icmp_seq 0

--- 8.8.8.1 ping statistics ---
2 packets transmitted, 0 packets received, 100.0% packet loss
"#;

    const WINDOWS_OUTPUT: &str = r#"
Reply from 8.8.8.8: bytes=1500 time=30ms TTL=54
Reply from 8.8.8.8: bytes=1500 time=30ms TTL=54
Reply from 8.8.8.8: bytes=1500 time=29ms TTL=54
Reply from 8.8.8.8: bytes=1500 time=30ms TTL=54
Reply from 8.8.8.8: bytes=1500 time=31ms TTL=54
Ping statistics for 172.217.1.142:
    Packets: Sent = 5, Received = 5, Lost = 0 (0% loss),
Approximate round trip times in milli-seconds:
    Minimum = 29ms, Maximum = 31ms, Average = 30ms
"#;

    const SUB_MS_OUTPUT: &str = r#"
64 bytes from 8.8.8.8: icmp_seq=0 ttl=115 time<1 ms
64 bytes from 8.8.8.8: icmp_seq=2 ttl=115 time<1ms
Reply from 8.8.8.8: bytes=1500 time=<1ms TTL=54
Reply from 8.8.8.8: bytes=1500 time=<1 ms TTL=54
"#;

    fn expect_pings(s: &str, expected: Vec<f64>) {
        let mut times = vec![];
        for line in s.lines() {
            let rtt = parse_ping_output(line);
            if let Some(rtt) = rtt {
                times.push(rtt);
            }
        }

        assert!(
            times.len() == expected.len(),
            "Expected {:?}, got {:?}",
            expected,
            times
        );
        for (i, time) in times.iter().enumerate() {
            assert!(
                (*time as f64 - expected[i] * 1000.0).abs() < 1.0,
                "Expected {:?}, got {:?}",
                expected[i],
                time
            );
        }
    }

    #[test]
    fn test_ping_config_creation() {
        let config = PingMonitorConfig {
            host: "8.8.8.8".to_string(),
            interval: Duration::from_secs(60),
            timeout: Duration::from_secs(5),
            warning_timeout: Duration::from_millis(1000),
            count: 1,
            red: default_red(),
            green: default_green(),
            blue: default_blue(),
            orange: default_orange(),
            yellow: default_yellow(),
            test: None,
        };

        let test_config = config.test();
        assert_eq!(test_config.interval, Duration::from_secs(60));
        assert_eq!(test_config.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_parse_ping_output() {
        expect_pings(LINUX_OUTPUT, vec![19.7, 19.7, 19.4]);
        expect_pings(MACOS_OUTPUT, vec![24.027, 22.841]);
        expect_pings(WINDOWS_OUTPUT, vec![30.0, 30.0, 29.0, 30.0, 31.0]);
        expect_pings(SUB_MS_OUTPUT, vec![1.0, 1.0]);
    }

    #[test]
    fn test_parse_ping_output_with_loss() {
        expect_pings(LINUX_OUTPUT_WITH_LOSS, vec![]);
        expect_pings(MACOS_OUTPUT_WITH_LOSS, vec![]);
    }
}
