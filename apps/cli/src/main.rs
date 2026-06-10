use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::atomic::Ordering;

mod commands;
mod engine;
mod utils;

use utils::printer::{JSON_MODE, VERBOSE_MODE};

#[derive(Parser)]
#[command(
    name = "cluaizd",
    about = "Cluaizd Enterprise CLI — full database control without a GUI",
    version = "0.1.0",
    long_about = None
)]
struct Cli {
    /// Path to the shard directory (for db/query commands)
    #[arg(short, long, default_value = "./data/shards", global = true)]
    path: PathBuf,

    /// Output results as structured JSON (for scripting and CI/CD pipelines)
    #[arg(long, global = true, default_value_t = false)]
    json: bool,

    /// Print verbose debug information
    #[arg(long, short = 'v', global = true, default_value_t = false)]
    verbose: bool,

    #[command(subcommand)]
    command: TopCommand,
}

#[derive(Subcommand)]
enum TopCommand {
    /// Manage cluaizd.toml configuration
    Config {
        #[command(subcommand)]
        action: ConfigCmd,
    },
    /// Manage the Cluaizd server daemon
    Server {
        #[command(subcommand)]
        action: ServerCmd,
    },
    /// Database shard operations (FFI direct — no server needed)
    Db {
        #[command(subcommand)]
        action: DbCmd,
    },
    /// WAL (Write-Ahead Log) inspection and diagnostics
    Wal {
        #[command(subcommand)]
        action: WalCmd,
    },
    /// Run a CDQL query directly via FFI (no server needed)
    Query {
        /// The CDQL query string to execute
        cdql: String,
    },
    /// Manage WASM DNA modules
    Dna {
        #[command(subcommand)]
        action: DnaCmd,
    },
    /// Start the Cluaizd server daemon directly
    Run {
        /// Path to the cluaizd-server binary
        #[arg(long, default_value = "./target/debug/cluaizd-server")]
        bin: PathBuf,
    },
}

#[derive(Subcommand)]
enum ConfigCmd {
    /// Print the full cluaizd.toml
    Show,
    /// Get a specific config value using dot-notation (e.g. `server.port`)
    Get { key: String },
    /// Set a config value using dot-notation (e.g. `server.port 8080`)
    /// If the value is omitted, an interactive dropdown will be shown for supported keys.
    Set { 
        key: String, 
        value: Option<String> 
    },
}

#[derive(Subcommand)]
enum ServerCmd {
    /// Start the server as a background daemon
    Start {
        /// Path to the cluaizd-server binary
        #[arg(long, default_value = "./target/debug/cluaizd-server")]
        bin: PathBuf,
    },
    /// Stop the running server daemon gracefully
    Stop,
    /// Show the server's current status (PID, port, log path)
    Status,
    /// Tail the server's log file
    Logs {
        /// Number of lines to show from the end of the log
        #[arg(long, default_value_t = 50)]
        lines: usize,
    },
}

#[derive(Subcommand)]
enum DbCmd {
    /// Check shard health and total neuron count
    Health,
    /// Inspect a specific neuron by UUID
    Inspect { id: String },
    /// Print detailed shard statistics (file size, tier breakdown)
    Stats,
    /// Create a live safe backup of the shard
    Backup {
        /// Destination directory for the backup
        dest: PathBuf,
    },
    /// Create a compacted copy of the shard (removes free pages)
    Compact,
}

#[derive(Subcommand)]
enum WalCmd {
    /// Inspect WAL files and print uncommitted entries
    Inspect {
        /// Maximum number of entries to display
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
}

#[derive(Subcommand)]
enum DnaCmd {
    /// List all active WASM DNAs in active_dnas/
    List,
    /// Deploy a compiled .wasm file to the active_dnas/ hot-reload cache
    Deploy {
        /// Path to the compiled .wasm file
        wasm: PathBuf,
    },
    /// Remove a DNA from the active_dnas/ cache
    Remove { name: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set global flags from parsed CLI args (atomic, no Mutex needed)
    JSON_MODE.store(cli.json, Ordering::Relaxed);
    VERBOSE_MODE.store(cli.verbose, Ordering::Relaxed);

    utils::printer::Printer::print_verbose(&format!("Shard path: {:?}", cli.path));
    utils::printer::Printer::print_verbose(&format!("JSON mode:  {}", cli.json));

    match cli.command {
        // ─── Config ─────────────────────────────────────────────────────────
        TopCommand::Config { action } => match action {
            ConfigCmd::Show               => commands::config::show(),
            ConfigCmd::Get { key }        => commands::config::get(&key),
            ConfigCmd::Set { key, value } => commands::config::set(&key, value),
        },

        // ─── Server ─────────────────────────────────────────────────────────
        TopCommand::Server { action } => match action {
            ServerCmd::Start { bin }   => commands::server::start(&bin),
            ServerCmd::Stop            => commands::server::stop(),
            ServerCmd::Status          => commands::server::status(),
            ServerCmd::Logs { lines }  => commands::server::logs(lines),
        },

        // ─── Database ───────────────────────────────────────────────────────
        TopCommand::Db { action } => match action {
            DbCmd::Health              => commands::health::run(&cli.path),
            DbCmd::Inspect { id }      => commands::inspect::run(&cli.path, &id),
            DbCmd::Stats               => commands::db_ops::stats(&cli.path),
            DbCmd::Backup { dest }     => commands::db_ops::backup(&cli.path, &dest),
            DbCmd::Compact             => commands::db_ops::compact(&cli.path),
        },

        // ─── WAL ────────────────────────────────────────────────────────────
        TopCommand::Wal { action } => match action {
            WalCmd::Inspect { limit }  => commands::wal_ops::inspect(limit),
        },

        // ─── Query ──────────────────────────────────────────────────────────
        TopCommand::Query { cdql }     => commands::query::run(&cli.path, &cdql),

        // ─── DNA ────────────────────────────────────────────────────────────
        TopCommand::Dna { action } => match action {
            DnaCmd::List               => commands::dna::list(),
            DnaCmd::Deploy { wasm }    => commands::dna::deploy(&wasm),
            DnaCmd::Remove { name }    => commands::dna::remove(&name),
        },

        // ─── Direct Server Run ──────────────────────────────────────────────
        TopCommand::Run { bin }        => commands::server::start(&bin),
    }
}
