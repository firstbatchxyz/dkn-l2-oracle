use crate::{commands, DriaOracle};
use clap::{Parser, Subcommand};
use eyre::Result;

#[derive(Subcommand, Debug)]
enum Commands {
    /// Adds files to myapp
    Balance,
    Register,
    Unregister,
    Claim,
    Rewards,
    Run,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

pub async fn cli(node: DriaOracle) -> Result<()> {
    let args = Args::parse();

    // TODO: add oracle kinds for relevant commands

    // TODO: add model parameter (for run & respond) only

    match args.command {
        Commands::Balance => commands::display_balance(node).await?,
        Commands::Register => commands::register(node).await?,
        Commands::Unregister => commands::unregister(node).await?,
        Commands::Claim => commands::claim_rewards(node).await?,
        Commands::Rewards => commands::display_rewards(node).await?,
        Commands::Run => commands::run_oracle(node).await?,
        // TODO: respond to latest available request
    };

    Ok(())
}
