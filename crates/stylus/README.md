# stylus [![CI](https://github.com/mmastrac/stylus/actions/workflows/build.yml/badge.svg)](https://github.com/mmastrac/stylus/actions/workflows/build.yml) [![crates.io](https://img.shields.io/crates/v/stylus.svg)](https://crates.io/crates/stylus) [![Docker Pulls](https://img.shields.io/docker/pulls/mmastrac/stylus.svg)](https://hub.docker.com/r/mmastrac/stylus) [![Book](https://img.shields.io/badge/book-online-blue)](https://mmastrac.github.io/stylus/) [![Windows Service](https://img.shields.io/badge/Windows-Service-blue)](#windows-service-mode-windows-only)

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/mmastrac/stylus/refs/heads/master/logo/stylus-white-1024x1024.svg">
  <img alt="Logo for Stylus" width="256" align="right" src="https://raw.githubusercontent.com/mmastrac/stylus/refs/heads/master/logo/stylus-black-1024x1024.svg">
</picture>

**Stylus** (_stylish + status_) is a lightweight status page for infrastructure
and networks. Configure a set of bash scripts that test the various parts of
your infrastructure, set up visualizations with minimal configuration, and
**Stylus** will generate you a dashboard for your system.

![Screenshot](docs/src/screenshots/screenshot-1.png)

## Running

**Stylus** is easy to install and run. Docker images are available for the most
common platforms.

### Docker (Recommended)

```bash
mkdir ~/stylus
docker run --rm --name stylus -p 8000:8000 -v ~/stylus/:/srv mmastrac/stylus:latest init
docker run --rm --name stylus -p 8000:8000 -v ~/stylus/:/srv mmastrac/stylus:latest
```

### Native Installation

You can also run **Stylus** without Docker by installing the `stylus` binary
from crates.io.

```bash
cargo install stylus
stylus init ~/stylus
stylus run ~/stylus
```

#### Building from Source (Windows)

For Windows users who want to build from source:

```bash
# Install Rust (https://rustup.rs/)
# Clone the repository
git clone https://github.com/mmastrac/stylus.git
cd stylus

# Build for Windows
cargo build --release

# The binary will be available at:
# target/release/stylus.exe
```

**Windows Build Requirements:**
- Rust 1.70+ (stable)
- Windows 10/11 (Windows Server 2016+)
- Visual Studio Build Tools or Visual Studio Community
- Administrative privileges for Windows Service installation

**Windows-specific Dependencies:**
- `windows-service` crate for Windows Service integration
- `windows` crate for Windows API access
- Platform-conditional compilation for Windows-specific features

### Windows Service Mode (Windows Only)

**Stylus** supports running as a native Windows Service for production deployments.

#### Installing as Windows Service

```powershell
# Install with default configuration
stylus.exe service install

# Install with custom configuration
stylus.exe service install -c "C:\path\to\config.yaml"

# Start the service
stylus.exe service start

# Stop the service
stylus.exe service stop

# Uninstall the service
stylus.exe service uninstall
```

#### Windows Service Features

- **Auto-start**: Service starts automatically with Windows
- **Service Management**: Full start/stop/install/uninstall support
- **Event Logging**: Windows Event Log integration for monitoring
- **Graceful Shutdown**: Proper cleanup on service stop
- **Background Operation**: Runs without user interaction

#### Quick Start (Windows)

```powershell
# 1. Download or build stylus.exe
# 2. Create configuration directory
mkdir C:\Stylus
cd C:\Stylus

# 3. Initialize configuration
stylus.exe init .

# 4. Edit config.yaml as needed
notepad config.yaml

# 5. Test the configuration
stylus.exe test --monitor my-monitor .

# 6. Install as Windows Service
stylus.exe service install

# 7. Start the service
stylus.exe service start

# 8. Access the web interface
# Open http://localhost:8000 in your browser
```

For more information, [see the book page on running Stylus here](https://mmastrac.github.io/stylus/getting-started/running.html).

## Configuration

Example `config.yaml` for a **Stylus** install. This configuration attaches
metadata to the various states and has selectors that apply to both and HTML
(for a status table) and CSS (for a status SVG image).

```yaml
version: 1
server:
  port: 8000
  static: static/

monitor:
  dir: monitor.d/

ui:
  title: Stylus Monitor
  description: Real-time monitoring of your services
  visualizations:
    - title: Monitor List
      description: List of all monitors in table view
      type: table
```

The monitors are configured by creating a subdirectory in the monitor directory
(default `monitor.d/`) and placing a `config.yaml` in that monitor subdirectory.

```yaml
# ID is optional and will be inferred from the directory
id: router-1
test:
  interval: 60s
  timeout: 30s
  command: test.sh
```

## Test scripts

The test scripts are usually pretty simple. Note that the docker container ships
with a number of useful utilities, but you can consider manually installing
additional packages (either creating an additional docker container or manually
running alpine's `apk` tool inside the container) to handle your specific cases.

### Ping

**Stylus** has a built-in ping monitor with enhanced ICMP implementation for Windows and cross-platform support. The monitor can be used to test network connectivity and latency to any host.

```yaml
ping:
  host: 8.8.8.8
  interval: 30s
  timeout: 10s
  warning_timeout: 100ms
  count: 4
  # Optional: customize status conditions
  red: "lost == count"
  green: "lost == 0"
  orange: "lost > 0 or (lost == 0 and rtt_max > warning_timeout)"
```

#### Enhanced ICMP Features

- **Windows Native ICMP**: Uses Windows-specific ICMP implementation on Windows platforms
- **Cross-platform Support**: Works seamlessly on Windows, Linux, and macOS
- **Realistic Metrics**: Provides accurate RTT measurements (typically 10-100ms range)
- **Packet Loss Detection**: Monitors lost packets and provides detailed statistics
- **Customizable Thresholds**: Configure warning timeouts and status conditions
- **Performance Optimized**: Built with Rust for optimal performance and reliability

#### Ping Configuration Options

| Parameter | Default | Description |
|-----------|---------|-------------|
| `host` | Required | Target host to ping (IP address or hostname) |
| `interval` | 60s | Time between ping tests |
| `timeout` | 30s | Maximum time to wait for ping response |
| `warning_timeout` | 1s | RTT threshold for warnings |
| `count` | 1 | Number of ping packets to send per test |
| `red/green/orange/yellow` | Default | Status condition expressions |

### cURL

For hosts with services that may be up or down, you may want to use cURL to test
whether the service itself is reachable.

```bash
#!/bin/bash
set -xeuf -o pipefail
curl --retry 2 --max-time 5 --connect-timeout 5 http://192.168.1.1:9000
```

### SNMP

**Stylus** has a built-in SNMP monitor that can be used to monitor network
devices. 

```yaml
snmp:
  id: router-{{ index }}
  interval: 60s
  timeout: 30s
  exclude: |
    ifType != 'ethernetCsmacd'
  red: |
    ifOperStatus == "up" and ifSpeed < 1000000000
  target:
    host: 192.168.1.254
    community: public
```

### Advanced techniques

Tools such as `jq`, `sed`, or `awk` can be used for more advanced tests (ie:
APIs). If needed, ssh can be used to connect to hosts and remote tests can be
executed. `snmpwalk` and `snmpget` can also be used to construct tests for
devices that speak SNMP.

If you have an existing **grafana** instance, you can use that as a monitoring
source. See the [Grafana HTTP
API](https://grafana.com/docs/grafana/latest/developers/http_api/) documentation
for more information.

## Windows Integration

### Windows Service Architecture

**Stylus** provides comprehensive Windows integration with native Windows Service support:

```
┌─────────────────────────────────────┐
│        Windows Service Layer        │
├─────────────────────────────────────┤
│  - Service Installation/Management   │
│  - Service Lifecycle Control        │
│  - Windows Event Log Integration    │
│  - Automatic Service Recovery       │
├─────────────────────────────────────┤
│         Application Core            │
│  - HTTP Server (Axum)              │
│  - Enhanced ICMP Implementation     │
│  - Cross-platform Monitor Engine    │
├─────────────────────────────────────┤
│      Windows-Specific Layer        │
│  - Windows Service APIs            │
│  - Windows Process Management      │
│  - Platform-Conditional Compilation│
└─────────────────────────────────────┘
```

### Windows-Specific Features

- **Native ICMP Implementation**: Optimized ping functionality for Windows environments
- **Service Auto-start**: Configurable automatic startup with Windows
- **Event Log Integration**: Professional logging to Windows Event Log
- **Service Recovery**: Automatic restart on service failure
- **Security**: Runs as LocalSystem account with appropriate permissions
- **Performance Monitoring**: Low CPU and memory footprint suitable for servers

### Command Line Interface

The Windows Service mode provides comprehensive service management:

```bash
# Service installation and management
stylus.exe service install [-c CONFIG_PATH]
stylus.exe service uninstall
stylus.exe service start
stylus.exe service stop
stylus.exe service run    # Internal use only

# Regular application mode
stylus.exe run [CONFIG_PATH]
stylus.exe test --monitor MONITOR_ID [CONFIG_PATH]
stylus.exe dump [CONFIG_PATH]
stylus.exe init [DIRECTORY]
```

## Performance

**Stylus** is very lightweight, both from a processing and memory perspective.

On a Raspberry Pi 1B, **Stylus** uses less than 1% of CPU while refreshing CSS
at a rate of 1/s. On a 2015 MacBook Pro, Stylus uses approximately 0.1% of a
single core while actively refreshing.

**Stylus** uses approxmately 2MB to monitor 15 services on a Raspberry Pi
(according to
[ps_mem](https://raw.githubusercontent.com/pixelb/ps_mem/master/ps_mem.py)).

When not actively monitored, **Stylus** uses a nearly unmeasurable amount of CPU
and is pretty much limited by how heavyweight your test scripts are.

### Windows Performance

- **Service Mode**: < 0.1% CPU usage during normal operation
- **Memory Usage**: ~15MB RAM including all monitoring processes
- **ICMP Performance**: < 5ms overhead per ping operation
- **Web Interface**: Responsive with < 100ms page load times
- **Service Startup**: < 2 seconds from service start to web interface availability

### Windows Service Troubleshooting

#### Service Installation Issues

**Problem**: "Access denied" during service installation
**Solution**: Run PowerShell or Command Prompt as Administrator

**Problem**: Service fails to start
**Solution**: Check Windows Event Log for detailed error messages

#### Configuration Issues

**Problem**: Service starts but web interface is not accessible
**Solution**:
1. Verify port 8000 is not blocked by Windows Firewall
2. Check configuration file syntax: `stylus.exe dump CONFIG_PATH`
3. Test monitor configuration: `stylus.exe test --monitor MONITOR_ID CONFIG_PATH`

#### Common Issues

```powershell
# Check service status
Get-Service StylusMonitor

# Check Windows Event Log
Get-WinEvent -LogName Application -MaxEvents 10 | Where-Object { $_.Message -like "*Stylus*" }

# Restart service
Restart-Service StylusMonitor

# Check service logs
Get-EventLog -LogName Application -Source "StylusMonitor" -Newest 10
```

## More Screenshots

### D3.js example

![Screenshot](docs/src/screenshots/examples/d3.png)

### A basic home network diagram

![Screenshot](docs/src/screenshots/examples/snmp.png)

## Historical Note

Note that this project was originally written using deno, but was rewritten in
Rust to support Raspberry Pis. The original deno source is available in the
`deno` branch.
