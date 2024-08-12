use crate::{contracts::OracleKind, DriaOracle};
use alloy::primitives::utils::format_ether;
use eyre::Result;

pub async fn register(node: &DriaOracle, kind: OracleKind) -> Result<()> {
    log::info!("Registering as a {}.", kind);
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
                "Approving {} tokens for {} registration.",
                kind,
                format_ether(difference)
            );
            node.approve(node.contract_addresses.registry, difference)
                .await?;
        } else {
            log::info!("Already approved enough tokens.");
        }

        // register
        log::info!("Registering.");
        node.register(kind).await?;
    }

    Ok(())
}

pub async fn unregister(node: &DriaOracle, kind: OracleKind) -> Result<()> {
    log::info!("Unregistering as {}.", kind);
    // check if not registered anyways
    if !node.is_registered(kind).await? {
        log::warn!("You are already not registered as a {}.", kind);
    } else {
        node.unregister(kind).await?;

        // transfer all allowance from registry back to oracle
        log::info!("Transferring all allowance back from registry.");
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

/// Displays the registration status of the oracle node for all oracle kinds.
pub async fn display_registrations(node: &DriaOracle) -> Result<()> {
    for kind in [OracleKind::Generator, OracleKind::Validator] {
        let is_registered = node.is_registered(kind).await?;
        log::info!("{}: {}", kind, is_registered);
    }

    Ok(())
}
