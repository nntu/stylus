use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Args {
    /// Pass multiple times to increase the level of verbosity (overwritten by STYLUS_LOG)
    #[arg(
        name = "verbose",
        long = "verbose",
        short = 'v',
        action = clap::ArgAction::Count,
        global = true
    )]
    verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Dumps the effective configuration without running
    Dump(DumpArgs),

    /// Runs the given test immediately and displays the status of the given monitor after it completes
    Test(TestArgs),

    /// Initialize a new stylus directory
    Init(InitArgs),

    /// Run stylus (default command)
    Run(RunArgs),

    #[cfg(windows)]
    /// Windows Service management commands
    Service(ServiceArgs),
}

#[derive(Debug, Parser)]
pub struct DumpArgs {
    /// The stylus directory containing the configuration file
    #[arg(name = "DIRECTORY", required_unless_present_any = ["force_container_path"])]
    pub directory: Option<PathBuf>,

    /// Advanced: if running a container, allows the container to override any path specified on the command line
    #[arg(env = "FORCE_CONTAINER_PATH", hide = true)]
    pub force_container_path: Option<PathBuf>,
}

#[derive(Debug, Parser)]
pub struct TestArgs {
    /// The test to run
    #[arg(short, long, required = true)]
    pub monitor: String,

    /// The stylus directory containing the configuration file
    #[arg(name = "DIRECTORY", required_unless_present_any = ["force_container_path"])]
    pub directory: Option<PathBuf>,

    /// Advanced: if running a container, allows the container to override any path specified on the command line
    #[arg(env = "FORCE_CONTAINER_PATH", hide = true)]
    pub force_container_path: Option<PathBuf>,

    /// Run as Windows Service (internal use)
    #[cfg(windows)]
    #[arg(long = "service", hide = true)]
    pub service_mode: bool,
}

#[derive(Debug, Parser)]
pub struct InitArgs {
    /// The directory to initialize (eg: `/srv/stylus`)
    #[arg(name = "DIRECTORY", required_unless_present_any = ["force_container_path"])]
    pub directory: Option<PathBuf>,

    /// Advanced: if running a container, allows the container to override any path specified on the command line
    #[arg(env = "FORCE_CONTAINER_PATH", hide = true)]
    pub force_container_path: Option<PathBuf>,

    /// Run as Windows Service (internal use)
    #[cfg(windows)]
    #[arg(long = "service", hide = true)]
    pub service_mode: bool,
}

#[derive(Debug, Parser)]
pub struct RunArgs {
    // TODO
    // /// Daemonize stylus and detact from the tty
    // #[arg(long, short)]
    // pub daemonize: bool,
    /// The stylus directory containing the configuration file
    #[arg(name = "DIRECTORY", required_unless_present_any = ["force_container_path"])]
    pub config: Option<PathBuf>,

    /// Dry run the configuration (everything except running the server)
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub dry_run: bool,

    /// Advanced: if running in a container, allows the container to override any port specified in config.yaml
    #[arg(env = "FORCE_CONTAINER_PORT", hide = true)]
    pub force_container_port: Option<u16>,

    /// Advanced: if running in a container, allows the container to specify that stylus should listen on the wildcard address
    #[arg(env = "FORCE_CONTAINER_LISTEN_ADDR", hide = true)]
    pub force_container_listen_addr: Option<String>,

    /// Advanced: if running a container, allows the container to override any path specified on the command line
    #[arg(env = "FORCE_CONTAINER_PATH", hide = true)]
    pub force_container_path: Option<PathBuf>,

    /// Run as Windows Service (internal use)
    #[cfg(windows)]
    #[arg(long = "service", hide = true)]
    pub service_mode: bool,
}

#[cfg(windows)]
#[derive(Debug, Subcommand)]
pub enum ServiceCommands {
    /// Install Windows Service
    Install {
        /// Configuration file path (optional)
        #[arg(short = 'c', long)]
        config: Option<String>,
    },

    /// Uninstall Windows Service
    Uninstall,

    /// Start Windows Service
    Start,

    /// Stop Windows Service
    Stop,

    /// Run as Windows Service (internal)
    Run,
}

#[cfg(windows)]
#[derive(Debug, Parser)]
pub struct ServiceArgs {
    #[command(subcommand)]
    pub command: ServiceCommands,
}