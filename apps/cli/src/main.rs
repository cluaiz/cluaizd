use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod commands;
mod engine;
mod utils;

#[derive(Parser)]
#[command(
    name = "cluaizd-cli",
    about = "Cluaizd Enterprise CLI — Control the database without a GUI",
    version = "0.1.0",
    long_about = None
)]
struct Cli {
    /// Path to the shard directory (for db/query commands)
    #[arg(short, long, default_value = "./data/shards", global = true)]
    path: PathBuf,

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
    /// Database shard operations (FFI direct)
    Db {
        #[command(subcommand)]
        action: DbCmd,
    },
    /// Run a CDQL query directly via FFI
    Query {
        /// The CDQL query string to execute
        cdql: String,
    },
    /// Manage WASM DNA modules
    Dna {
        #[command(subcommand)]
        action: DnaCmd,
    },
}

#[derive(Subcommand)]
enum ConfigCmd {
    /// Print the full cluaizd.toml
    Show,
    /// Get a specific config value (e.g. `server.port`)
    Get { key: String },
    /// Set a config value (e.g. `server.port 8080`)
    Set { key: String, value: String },
}

#[derive(Subcommand)]
enum ServerCmd {
    /// Start the server as a background daemon
    Start {
        /// Path to the cluaizd-server binary
        #[arg(long, default_value = "./target/debug/server")]
        bin: PathBuf,
    },
    /// Stop the running server daemon
    Stop,
    /// Show the server's current status
    Status,
}

#[derive(Subcommand)]
enum DbCmd {
    /// Check shard health and neuron count
    Health,
    /// Inspect a specific neuron by UUID
    Inspect { id: String },
}

#[derive(Subcommand)]
enum DnaCmd {
    /// List all active WASM DNAs
    List,
    /// Deploy a compiled .wasm file to active_dnas/
    Deploy {
        /// Path to the .wasm file
        wasm: PathBuf,
    },
    /// Remove a DNA from active_dnas/
    Remove { name: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        // ─── Config ───────────────────────────────────────────────────────────
        TopCommand::Config { action } => match action {
            ConfigCmd::Show => commands::config::show(),
            ConfigCmd::Get { key } => commands::config::get(&key),
            ConfigCmd::Set { key, value } => commands::config::set(&key, &value),
        },

        // ─── Server ───────────────────────────────────────────────────────────
        TopCommand::Server { action } => match action {
            ServerCmd::Start { bin } => commands::server::start(&bin),
            ServerCmd::Stop => commands::server::stop(),
            ServerCmd::Status => commands::server::status(),
        },

        // ─── Database ─────────────────────────────────────────────────────────
        TopCommand::Db { action } => match action {
            DbCmd::Health => commands::health::run(&cli.path),
            DbCmd::Inspect { id } => commands::inspect::run(&cli.path, &id),
        },

        // ─── Query ────────────────────────────────────────────────────────────
        TopCommand::Query { cdql } => commands::query::run(&cli.path, &cdql),

        // ─── DNA ──────────────────────────────────────────────────────────────
        TopCommand::Dna { action } => match action {
            DnaCmd::List => commands::dna::list(),
            DnaCmd::Deploy { wasm } => commands::dna::deploy(&wasm),
            DnaCmd::Remove { name } => commands::dna::remove(&name),
        },
    }
}
