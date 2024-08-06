use crate::DriaOracle;
use alloy::primitives::utils::format_ether;
use eyre::Result;

pub async fn register(node: DriaOracle) -> Result<()> {
    // check if registered already
    if node.is_registered().await? {
        log::warn!("You are already registered.");
    } else {
        // calculate the required approval for registration
        let (stake_amount, _, _) = node.registry_stake_amount().await?;
        let (allowance, _, _) = node
            .allowance(node.address, node.contract_addresses.registry)
            .await?;

        // approve if necessary
        if allowance < stake_amount {
            let difference = stake_amount - allowance;
            log::info!(
                "Approving {} tokens for registration.",
                format_ether(difference)
            );
            node.approve(node.contract_addresses.registry, difference)
                .await?;
        }

        // register
        node.register().await?;
    }

    Ok(())
}

pub async fn unregister(node: DriaOracle) -> Result<()> {
    // check if not registered anyways
    if !node.is_registered().await? {
        log::warn!("You are already not registered.");
    } else {
        node.unregister().await?;

        // transfer all allowance from registry back to oracle
        let (allowance, _, _) = node
            .allowance(node.contract_addresses.registry, node.address)
            .await?;
        node.transfer_from(node.contract_addresses.registry, node.address, allowance)
            .await?;
    }
    Ok(())
}
