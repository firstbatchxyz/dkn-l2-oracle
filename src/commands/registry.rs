use crate::DriaOracle;
use eyre::Result;

pub async fn register(node: DriaOracle) -> Result<()> {
    // check if registered already
    if node.is_registered().await? {
        log::warn!("You are already registered.");
    } else {
        // calculate & approve the required approval for registration
        let stake_amount = node.registry_stake_amount().await?.0;
        let allowance = node
            .allowance(node.address, node.contract_addresses.registry)
            .await?;
        if allowance < stake_amount {
            let difference = stake_amount - allowance;
            node.approve(node.contract_addresses.registry, difference)
                .await?;
        } else {
            // TODO: log
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
        node.transfer_from(node.contract_addresses.registry, node.address, allowance)
            .await?;
    }
    Ok(())
}
