# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Stylus** is a lightweight, Rust-based status page and monitoring system for home infrastructure. It monitors various services through configurable monitors and displays real-time status through a web interface with built-in React UI.

## Build and Development Commands

### Core Commands
```bash
# Development build
cargo build --bin stylus

# Release build
cargo build --release --bin stylus

# Run the application
cargo run -- <command> [args]

# Initialize new configuration
cargo run -- init <path>

# Start monitoring server
cargo run -- run <path>

# Test specific monitor
cargo run -- test <path> <monitor_id>

# Dump configuration for debugging
cargo run -- dump <path>
```

### Using Just (Recommended)
The project uses `just` for common development tasks:

```bash
# Build debug version
just build-debug

# Run all tests
just test

# Run only Rust tests
just test-rust

# Run only CLI tests
just test-cli

# Start development server with live UI editing
just dev

# Build UI bundle from source
just bundle

# Check TypeScript compilation
just ts-check

# Clean compiled UI assets
just clean-bundle
```

### UI Development Commands
```bash
# Build with UI from source (for development)
cargo build --features=from-source-auto

# Build with UI from source (always rebuild)
cargo build --features=from-source-always

# Build without UI (core functionality only)
cargo build --no-default-features
```

## Architecture Overview

### Workspace Structure
- **`crates/stylus/`**: Main application binary and core monitoring logic
- **`crates/stylus-ui/`**: Built-in web UI assets (React, TypeScript, CSS)

### Core Components

#### 1. Configuration System (`crates/stylus/src/config/`)
- **`mod.rs`**: Configuration loading, path resolution, command validation
- **`structs.rs`**: Configuration structs and serialization
- **`args.rs`**: CLI argument parsing

**Key Pattern**: YAML-based configuration with hierarchical structure. Main `config.yaml` + `monitor.d/` subdirectories for individual monitors.

#### 2. Monitoring System (`crates/stylus/src/monitor.rs`, `crates/stylus/src/worker/`)
- **Actor-like Pattern**: Each monitor runs in its own thread
- **Message Processing**: Handles logs, metadata, and termination signals
- **State Management**: Thread-safe status tracking using `SharedMut` (keepcalm crate)

**Monitor Types**:
- **Script Monitors**: Execute arbitrary shell scripts
- **Group Monitors**: Template-based multi-monitor generation
- **Built-in Monitors**: Ping (`monitors/ping.rs`), SNMP (`monitors/snmp.rs`)

#### 3. HTTP Server (`crates/stylus/src/http.rs`)
- **Framework**: Axum async web server
- **Key Endpoints**:
  - `/` - Main dashboard (HTML)
  - `/status.json` - Real-time status data
  - `/config.json` - Configuration data
  - `/log/:monitor_id` - Monitor logs
  - `/style.css` - Dynamic CSS generation
- **Features**: ETag caching, path validation, static file serving

#### 4. UI System (`crates/stylus-ui/`)
- **Build System**: Conditional compilation with Deno
- **Features**:
  - `use_files` (default): Pre-built assets
  - `no_use_files`: Build from source
- **Components**: React app with TypeScript, real-time updates, multiple visualizations

#### 5. Status Management (`crates/stylus/src/status.rs`)
- **States**: `Blank`, `Green`, `Yellow`, `Red`, `Blue`, `Orange`
- **MonitorState**: Individual monitor state with logs and metadata
- **CSS Integration**: Dynamic styling based on monitor status

### Cross-Platform Considerations
The project supports Windows, Linux, and macOS with:
- **Command Resolution**: Uses `where` (Windows) or `which` (Unix) for PATH detection
- **Monitor Execution**: Platform-specific command syntax (e.g., ping arguments)
- **Shell Wrapping**: `cmd /C` (Windows) vs `/bin/sh -c` (Unix) for complex commands

## Testing

### Test Structure
- **Unit Tests**: Inline `#[test]` modules in source files
- **Integration Tests**: `src/testcases/` directory with sample configurations
- **CLI Tests**: `tests/` directory with command-line test scenarios

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test module
cargo test config::tests

# Run tests with output
cargo test -- --nocapture

# Run CLI integration tests
just test-cli
```

## Key Configuration Files

### Main Configuration (`config.yaml`)
```yaml
version: 1
server:
  port: 8000
  listen_addr: 0.0.0.0
  static: static/
monitor:
  dir: monitor.d
ui:
  title: "Stylus Monitor"
  visualizations:
    - type: table
    - type: isoflow
css:
  metadata: {}
  rules: []
```

### Monitor Configuration (`monitor.d/*/config.yaml`)
- **Script Monitor**: `test.command` and `test.args`
- **Ping Monitor**: `ping.host`, `ping.interval`, `ping.timeout`
- **SNMP Monitor**: `snmp.target`, `snmp.community`, `snmp.version`
- **Group Monitor**: `group.axes` for template-based generation

## Development Workflow

### Adding New Monitor Types
1. Create configuration struct in `config/structs.rs`
2. Update `MonitorDirRootConfig` enum
3. Implement `MonitorMessageProcessor` trait
4. Add monitor implementation in `monitors/` directory
5. Update `test()` method to return `MonitorDirTestConfig`

### UI Development
1. Use development mode: `just dev` or `cargo run --no-default-features`
2. Edit TypeScript/React files in `crates/stylus-ui/web/src/`
3. Build with `--features=from-source-auto` for changes
4. Check browser console for errors

### Configuration Debugging
```bash
# Check parsed configuration
cargo run -- dump /path/to/config

# Test specific monitor
cargo run -- test /path/to/config monitor-name

# Verbose logging
RUST_LOG=debug cargo run -- run /path/to/config
```

## Important Implementation Details

### Monitor Thread Management
- Each monitor runs in a separate thread with controlled lifecycle
- Workers handle process execution with timeouts and resource limits
- State updates are thread-safe through `SharedMut` abstraction

### Asset Building System
- Two build modes: pre-compiled assets (default) vs from-source
- Deno handles TypeScript compilation and CSS inlining
- Source maps generated for debugging
- Compression applied where available (skipped on Windows)

### Security Considerations
- Path validation prevents directory traversal in static file serving
- Monitor processes run with limited environment and strict timeouts
- Configuration parsing with strict type checking and validation

### Performance Optimizations
- ETag-based caching for static assets
- Efficient state synchronization using `keepcalm`
- Minimal memory footprint (~2MB for 15 services)
- Smart polling with configurable intervals