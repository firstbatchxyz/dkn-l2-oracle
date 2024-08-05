use alloy::primitives::utils::format_ether;
use eyre::Result;

use crate::DriaOracle;

pub async fn display_balance(node: DriaOracle) -> Result<()> {
    let balances = node.balances().await?;
    log::info!("Your balances:");
    log::info!("{} {}", format_ether(balances.0 .0), balances.0 .1);
    log::info!("{} {}", format_ether(balances.1 .0), balances.1 .1);

    Ok(())
}

/// Show the amount of claimable rewards
pub async fn display_rewards(node: DriaOracle) -> Result<()> {
    let allowance = node
        .allowance(node.contract_addresses.coordinator, node.address)
        .await?;

    // TODO:

    Ok(())
}

/// Claim rewards
pub async fn claim_rewards(node: DriaOracle) -> Result<()> {
    // TODO: !!!

    Ok(())
}
