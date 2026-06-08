use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cli")]
#[command(about = "Cluaizd CLUAIZD Administration CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Check the health of the CLUAIZD server
    Health,
    /// Inspect the contents of a specific neuron
    Inspect {
        /// The UUID of the neuron to inspect
        id: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = reqwest::Client::new();
    let base_url = "http://127.0.0.1:7331";

    match &cli.command {
        Commands::Health => {
            // Check if server is reachable
            match client.get(format!("{}/neuron/00000000-0000-0000-0000-000000000000", base_url)).send().await {
                Ok(_) => println!("Status: CLUAIZD is healthy and running on {}.", base_url),
                Err(e) => eprintln!("Error: Could not connect to CLUAIZD at {}. Is the server running? ({})", base_url, e),
            }
        }
        Commands::Inspect { id } => {
            let url = format!("{}/neuron/{}", base_url, id);
            match client.get(&url).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let text = resp.text().await?;
                        println!("Neuron {}:\n{}", id, text);
                    } else {
                        let status = resp.status();
                        let text = resp.text().await.unwrap_or_default();
                        eprintln!("Failed to inspect neuron. Status: {} - {}", status, text);
                    }
                }
                Err(e) => eprintln!("Error: Failed to connect to CLUAIZD. ({})", e),
            }
        }
    }

    Ok(())
}

