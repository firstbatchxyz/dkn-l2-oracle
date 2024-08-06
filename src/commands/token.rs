use alloy::primitives::utils::format_ether;
use eyre::Result;

use crate::DriaOracle;

/// Display token balances
pub async fn display_balance(node: DriaOracle) -> Result<()> {
    let balances = node.balances().await?;

    log::info!("Your balances:");
    log::info!(
        "{} {} {}",
        format_ether(balances[0].0),
        balances[0].1,
        balances[0].2.map(|s| s.to_string()).unwrap_or_default()
    );
    log::info!(
        "{} {} {}",
        format_ether(balances[1].0),
        balances[1].1,
        balances[1].2.map(|s| s.to_string()).unwrap_or_default()
    );

    Ok(())
}

/// Show the amount of claimable rewards
pub async fn display_rewards(node: DriaOracle) -> Result<()> {
    let (allowance, symbol, addr) = node
        .allowance(node.contract_addresses.coordinator, node.address)
        .await?;

    log::info!("Claimable rewards:");
    log::info!(
        "{} {} {}",
        format_ether(allowance),
        symbol,
        addr.map(|s| s.to_string()).unwrap_or_default()
    );

    if allowance.is_zero() {
        log::warn!("You have no claimable rewards!");
    }

    Ok(())
}

/// Claim rewards
pub async fn claim_rewards(node: DriaOracle) -> Result<()> {
    // get allowance
    let (allowance, _, _) = node
        .allowance(node.contract_addresses.coordinator, node.address)
        .await?;

    // check if there are rewards to claim
    if allowance.is_zero() {
        log::warn!("No rewards to claim.");
    } else {
        // transfer rewards
        node.transfer_from(node.contract_addresses.coordinator, node.address, allowance)
            .await?;
    }

    Ok(())
}
