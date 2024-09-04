use crate::{contracts::OracleKind, DriaOracle};
use alloy::primitives::utils::format_ether;
use eyre::Result;

/// Registers the oracle node as an oracle for the given kind.
///
/// - If the node is already registered, it will do nothing.
/// - If the node is not registered, it will approve the required amount of tokens
/// to the registry and then register the node.
pub async fn register(node: &DriaOracle, kind: OracleKind) -> Result<()> {
    log::info!("Registering as a {}.", kind);
    // check if registered already
    if node.is_registered(kind).await? {
        log::warn!("You are already registered as a {}.", kind);
    } else {
        // calculate the required approval for registration
        let stake = node.registry_stake_amount(kind).await?;
        let allowance = node
            .allowance(node.address(), node.addresses.registry)
            .await?;

        // approve if necessary
        if allowance.amount < stake.amount {
            let difference = stake.amount - allowance.amount;
            log::info!(
                "Approving {} tokens for {} registration.",
                format_ether(difference),
                kind
            );

            // check balance
            let balance = node.get_token_balance(node.address()).await?;
            if balance.amount < difference {
                return Err(eyre::eyre!(
                    "Not enough balance to approve. (have: {}, required: {})",
                    balance,
                    difference
                ));
            }

            // approve the difference
            node.approve(node.addresses.registry, difference).await?;
        } else {
            log::info!("Already approved enough tokens.");
        }

        // register
        log::info!("Registering.");
        node.register(kind).await?;
    }

    Ok(())
}

/// Unregisters the oracle node as an oracle for the given kind.
///
/// - If the node is not registered, it will do nothing.
/// - If the node is registered, it will unregister the node and transfer all allowance
/// from the registry back to the oracle.
pub async fn unregister(node: &DriaOracle, kind: OracleKind) -> Result<()> {
    log::info!("Unregistering as {}.", kind);
    // check if not registered anyways
    if !node.is_registered(kind).await? {
        log::warn!("You are already not registered as a {}.", kind);
    } else {
        node.unregister(kind).await?;

        // transfer all allowance from registry back to oracle
        let allowance = node
            .allowance(node.addresses.registry, node.address())
            .await?;
        log::info!(
            "Transferring all allowance ({}) back from registry.",
            allowance
        );
        node.transfer_from(node.addresses.registry, node.address(), allowance.amount)
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
