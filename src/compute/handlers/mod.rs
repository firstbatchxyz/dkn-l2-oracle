use crate::{
    contracts::{OracleCoordinator::StatusUpdate, OracleKind, TaskStatus},
    DriaOracle,
};
use alloy::rpc::types::TransactionReceipt;
use dkn_workflows::DriaWorkflowsConfig;
use eyre::Result;

mod generation;
use generation::*;

mod validation;
use validation::*;

/// Handles a task request.
///
/// - Generation tasks are forwarded to `handle_generation`
/// - Validation tasks are forwarded to `handle_validation`
///
///
pub async fn handle_request(
    node: &DriaOracle,
    kinds: &[OracleKind],
    model_config: &DriaWorkflowsConfig,
    event: StatusUpdate,
) -> Result<Option<TransactionReceipt>> {
    log::debug!("Received event for task {} ()", event.taskId);

    let response_tx_hash = match TaskStatus::try_from(event.statusAfter)? {
        TaskStatus::PendingGeneration => {
            if kinds.contains(&OracleKind::Generator) {
                handle_generation(node, model_config, event.taskId, event.protocol).await?
            } else {
                log::debug!(
                    "Ignoring generation task {} as you are not generator.",
                    event.taskId
                );
                return Ok(None);
            }
        }
        TaskStatus::PendingValidation => {
            if kinds.contains(&OracleKind::Validator) {
                handle_validation(node, model_config, event.taskId).await?
            } else {
                log::debug!(
                    "Ignoring generation task {} as you are not validator.",
                    event.taskId
                );
                return Ok(None);
            }
        }
        TaskStatus::None => {
            log::error!("None status received in an event: {}", event.taskId);
            return Ok(None);
        }
        TaskStatus::Completed => {
            log::debug!("Task {} is completed.", event.taskId);
            return Ok(None);
        }
    };

    Ok(response_tx_hash)
}
