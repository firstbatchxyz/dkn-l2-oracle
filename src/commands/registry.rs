use crate::{contracts::OracleKind, DriaOracle};
use alloy::primitives::utils::format_ether;
use eyre::Result;

pub async fn register(node: DriaOracle, kind: OracleKind) -> Result<()> {
    // check if registered already
    if node.is_registered(kind).await? {
        log::warn!("You are already registered as a {}.", kind);
    } else {
        // calculate the required approval for registration
        let stake = node.registry_stake_amount(kind).await?;
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
        node.register(kind).await?;
    }

    Ok(())
}

pub async fn unregister(node: DriaOracle, kind: OracleKind) -> Result<()> {
    // check if not registered anyways
    if !node.is_registered(kind).await? {
        log::warn!("You are already not registered.");
    } else {
        node.unregister(kind).await?;

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

// TODO: display all registry types
pub async fn list_registrations(node: DriaOracle) -> Result<()> {
    // TODO: loop over all oracle kinds and display registry status
    Ok(())
}
