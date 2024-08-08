use crate::{commands, DriaOracle};
use clap::{Args, Parser, Subcommand};
use eyre::Result;

#[derive(Args)]
struct KindArgs {
    #[arg(help = "The oracle kinds to register as.", required = true)]
    kinds: Vec<String>,
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

    #[arg(short, long, env = "RPC_URL")]
    rpc_url: String,
}

pub async fn cli(node: DriaOracle) -> Result<()> {
    let matched_commands = Cli::parse().command;

    // TODO: parse params and create node here

    // TODO: add model parameter (for run & respond) only

    match matched_commands {
        Commands::Balance => commands::display_balance(&node).await?,
        Commands::Register(arg) => {
            for kind in arg.kinds {
                commands::register(&node, kind.try_into()?).await?
            }
        }
        Commands::Unregister(arg) => {
            for kind in arg.kinds {
                commands::unregister(&node, kind.try_into()?).await?;
            }
        }
        Commands::Registrations => commands::display_registrations(&node).await?,
        Commands::Claim => commands::claim_rewards(&node).await?,
        Commands::Rewards => commands::display_rewards(&node).await?,
        Commands::Run => commands::run_oracle(&node, vec![]).await?, // TODO: !!
                                                                     // TODO: respond to latest available request
    };

    Ok(())
}
