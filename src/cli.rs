use crate::{commands, contracts::OracleKind, DriaOracle};
use alloy::primitives::U256;
use clap::{Parser, Subcommand};
use eyre::{eyre, Result};
use ollama_workflows::Model;

/// `value_parser` to parse a `str` to `OracleKind`.
fn parse_oracle_kind(value: &str) -> Result<OracleKind> {
    OracleKind::try_from(value)
}

/// `value_parser` to parse a `str` to `OracleKind`.
fn parse_model(value: &str) -> Result<Model> {
    Model::try_from(value.to_string()).map_err(|e| eyre!(e))
}

// https://docs.rs/clap/latest/clap/_derive/index.html#arg-attributes
#[derive(Subcommand)]
enum Commands {
    /// See the current balance of the oracle node.
    Balance,
    /// Register oracle as a specific oracle kind.
    Register {
        #[arg(help = "The oracle kinds to register as.", required = true, value_parser=parse_oracle_kind)]
        kinds: Vec<OracleKind>,
    },
    /// Unregister oracle as a specific oracle kind.
    Unregister {
        #[arg(help = "The oracle kinds to unregister as.", required = true, value_parser=parse_oracle_kind)]
        kinds: Vec<OracleKind>,
    },
    /// See all registrations.
    Registrations,
    /// Claim rewards from the coordinator.
    Claim,
    /// See claimable rewards from the coordinator.
    Rewards,
    /// Launch the oracle node.
    Run {
        #[arg(help = "The oracle kinds to handle tasks as.", required = true, value_parser=parse_oracle_kind)]
        kinds: Vec<OracleKind>,
        #[arg(short, long, help = "The models to serve.", required = true, value_parser=parse_model)]
        models: Vec<Model>,
    },
    /// View status of a given task.
    View { task_id: U256 },
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

    match matched_commands {
        Commands::Balance => commands::display_balance(&node).await?,
        Commands::Register { kinds } => {
            for kind in kinds {
                commands::register(&node, kind).await?
            }
        }
        Commands::Unregister { kinds } => {
            for kind in kinds {
                commands::unregister(&node, kind).await?;
            }
        }
        Commands::Registrations => commands::display_registrations(&node).await?,
        Commands::Claim => commands::claim_rewards(&node).await?,
        Commands::Rewards => commands::display_rewards(&node).await?,
        Commands::Run { kinds, models } => commands::run_oracle(&node, kinds, models).await?,
        Commands::View { task_id } => commands::view_task(&node, task_id).await?,
        // TODO: add "respond to latest available request" command
    };

    Ok(())
}
