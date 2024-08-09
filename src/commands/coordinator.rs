use crate::{
    compute,
    contracts::{OracleKind, TaskStatus},
    DriaOracle,
};
use alloy::primitives::U256;
use eyre::{eyre, Result};
use futures_util::StreamExt;

// TODO: add cancellation here
/// Runs the main loop of the oracle node.
pub async fn run_oracle(node: &DriaOracle, kinds: Vec<OracleKind>) -> Result<()> {
    // make sure we are registered
    for kind in &kinds {
        if !node.is_registered(*kind).await? {
            return Err(eyre!("You need to register as {} first.", kind))?;
        }
    }

    let task_poller = node.subscribe_to_tasks().await?;
    log::info!(
        "Subscribed to LLMOracleCoordinator at {}",
        node.contract_addresses.coordinator
    );

    let is_generator = kinds.contains(&OracleKind::Generator);
    let is_validator = kinds.contains(&OracleKind::Validator);

    task_poller
        .into_stream()
        .for_each(|log| async {
            match log {
                Ok((event, log)) => {
                    log::debug!(
                        "Received event for tx: {}",
                        log.transaction_hash.unwrap_or_default()
                    );
                    log::info!("Received event for task: {}", event.taskId);

                    // handle the event based on task status
                    let response_tx_hash = match TaskStatus::try_from(event.statusAfter)
                        .unwrap_or_else(|e| {
                            log::error!("Could not parse task status: {}", e);
                            TaskStatus::default()
                        }) {
                        TaskStatus::PendingGeneration => {
                            if is_generator {
                                compute::handle_generation(node, event.taskId).await
                            } else {
                                return;
                            }
                        }
                        TaskStatus::PendingValidation => {
                            if is_validator {
                                compute::handle_validation(node, event.taskId).await
                            } else {
                                return;
                            }
                        }
                        _ => {
                            log::debug!(
                                "Ignoring task {} with status: {}",
                                event.taskId,
                                event.statusAfter
                            );
                            return;
                        }
                    };

                    match response_tx_hash {
                        Ok(tx_hash) => {
                            log::info!(
                                "Task {} processed successfully. (tx: {})",
                                event.taskId,
                                tx_hash
                            )
                        }
                        Err(e) => log::error!("Could not process task: {:?}", e),
                    }
                }
                Err(e) => log::error!("Could not handle event: {:?}", e),
            }
        })
        .await;

    Ok(())
}

pub async fn view_task(node: &DriaOracle, task_id: U256) -> Result<()> {
    log::info!("Displaying task {}.", task_id);

    let (request, responses, validations) = node.get_task(task_id).await?;

    log::info!("Request: {}", request.status);
    Ok(())
}
