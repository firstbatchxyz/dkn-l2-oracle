use crate::{commands, DriaOracle};
use clap::{Args, Parser, Subcommand};
use eyre::Result;

#[derive(Args)]
struct KindArgs {
    // TODO: add example & help
    kind: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// See the current balance of the oracle node.
    Balance,
    /// Register oracle as a specific oracle kind.
    Register(KindArgs),
    /// Unregister oracle as a specific oracle kind.
    Unregister(KindArgs),
    /// See all registrations.
    Registrations,
    /// Claim rewards from the coordinator.
    Claim,
    /// See claimable rewards from the coordinator.
    Rewards,
    /// Launch the oracle node.
    Run,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

pub async fn cli(node: DriaOracle) -> Result<()> {
    let matched_commands = Cli::parse().command;

    // TODO: add oracle kinds for relevant commands

    // TODO: add model parameter (for run & respond) only

    match matched_commands {
        Commands::Balance => commands::display_balance(node).await?,
        Commands::Register(arg) => {
            commands::register(node, arg.kind[0].clone().try_into()?).await?
        }
        Commands::Unregister(arg) => {
            commands::unregister(node, arg.kind[0].clone().try_into()?).await?
        }
        Commands::Registrations => commands::registrations(node).await?,
        Commands::Claim => commands::claim_rewards(node).await?,
        Commands::Rewards => commands::display_rewards(node).await?,
        Commands::Run => commands::run_oracle(node, vec![]).await?, // TODO: !!
                                                                    // TODO: respond to latest available request
    };

    Ok(())
}
