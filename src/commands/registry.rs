use crate::DriaOracle;
use alloy::primitives::utils::format_ether;
use eyre::Result;

pub async fn register(node: DriaOracle) -> Result<()> {
    // check if registered already
    if node.is_registered().await? {
        log::warn!("You are already registered.");
    } else {
        // calculate the required approval for registration
        let stake = node.registry_stake_amount().await?;
        let allowance = node
            .allowance(node.address, node.contract_addresses.registry)
            .await?;

        // approve if necessary
        if allowance.amount < stake.amount {
            let difference = stake.amount - allowance.amount;
            log::info!(
                "Approving {} tokens for registration.",
                format_ether(difference)
            );
            node.approve(node.contract_addresses.registry, difference)
                .await?;
        } else {
            log::info!("Already approved enough tokens for registration.");
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
        let allowance = node
            .allowance(node.contract_addresses.registry, node.address)
            .await?;
        node.transfer_from(
            node.contract_addresses.registry,
            node.address,
            allowance.amount,
        )
        .await?;
    }
    Ok(())
}
