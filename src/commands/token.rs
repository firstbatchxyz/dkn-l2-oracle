use crate::DriaOracle;
use eyre::Result;

/// Display token balances
pub async fn display_balance(node: &DriaOracle) -> Result<()> {
    let eth_balance = node.get_native_balance(node.address()).await?;
    let token_balance = node.get_token_balance(node.address()).await?;

    log::info!("Your balances:");
    for balance in [eth_balance, token_balance].iter() {
        log::info!("{}", balance);
    }

    Ok(())
}

/// Show the amount of claimable rewards
pub async fn display_rewards(node: &DriaOracle) -> Result<()> {
    let allowance = node
        .allowance(node.addresses.coordinator, node.address())
        .await?;

    log::info!("Claimable rewards:");
    log::info!("{} ", allowance);

    if allowance.amount.is_zero() {
        log::warn!("You have no claimable rewards!");
    }

    Ok(())
}

/// Claim rewards
pub async fn claim_rewards(node: &DriaOracle) -> Result<()> {
    // get allowance
    let allowance = node
        .allowance(node.addresses.coordinator, node.address())
        .await?;

    // check if there are rewards to claim
    if allowance.amount.is_zero() {
        log::warn!("No rewards to claim.");
    } else {
        // transfer rewards
        node.transfer_from(node.addresses.coordinator, node.address(), allowance.amount)
            .await?;
    }

    Ok(())
}
