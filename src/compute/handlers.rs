use super::{ModelConfig, WorkflowsExt};
use crate::{
    contracts::{
        bytes32_to_string, bytes_to_string, OracleCoordinator::StatusUpdate, OracleKind, TaskStatus,
    },
    DriaOracle,
};
use alloy::{
    primitives::{utils::parse_ether, Bytes, FixedBytes, U256},
    rpc::types::TransactionReceipt,
};
use eyre::{eyre, Context, Result};
use ollama_workflows::Executor;

use super::mine_nonce;

pub async fn handle_request(
    node: &DriaOracle,
    kinds: &[OracleKind],
    model_config: &ModelConfig,
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
            unreachable!("TaskStatus::None is impossible to receive in an event");
        }
        TaskStatus::Completed => {
            log::debug!("Task {} is completed.", event.taskId);
            return Ok(None);
        }
    };

    Ok(response_tx_hash)
}

/// Handles a generation request.
async fn handle_generation(
    node: &DriaOracle,
    models: &ModelConfig,
    task_id: U256,
    protocol: FixedBytes<32>,
) -> Result<Option<TransactionReceipt>> {
    let responses = node.get_task_responses(task_id).await?;
    if responses.iter().any(|r| r.responder == node.address()) {
        log::debug!("Already responded to {} with generation", task_id);
        return Ok(None);
    }

    let request = node
        .get_task_request(task_id)
        .await
        .wrap_err("Could not get task")?;

    // choose model based on the request
    let models_string = bytes_to_string(&request.models)?;
    let (_, model) = models.get_any_matching_model_from_csv(&models_string)?;
    log::debug!("Using model: {} from {}", model, models_string);

    // execute task
    let protocol_string = bytes32_to_string(&protocol)?;
    let executor = Executor::new(model);
    let (output_str, metadata_str) = executor
        .execute_raw(&request.input, &protocol_string)
        .await?;
    log::debug!("Output: {}", output_str);
    let output = Bytes::from_iter(output_str.as_bytes());
    let metadata = Bytes::from_iter(metadata_str.as_bytes());

    // do the Arweave trick for large inputs

    // mine nonce
    let nonce = mine_nonce(
        request.parameters.difficulty,
        &request.requester,
        &node.address(),
        &request.input,
        &task_id,
    )
    .0;

    // respond
    let tx_hash = node
        .respond_generation(task_id, output, metadata, nonce)
        .await?;
    Ok(Some(tx_hash))
}

/// Handles a validation request.
#[allow(unused)]
async fn handle_validation(
    node: &DriaOracle,
    models: &ModelConfig,
    task_id: U256,
) -> Result<Option<TransactionReceipt>> {
    // check if already responded as generator, because we cant validate our own answer
    let responses = node.get_task_responses(task_id).await?;
    if responses.iter().any(|r| r.responder == node.address()) {
        log::debug!(
            "Cant validate {} with your own generation response",
            task_id
        );
        return Ok(None);
    }

    // check if we have validated anyways
    let validations = node.get_task_validations(task_id).await?;
    if validations.iter().any(|v| v.validator == node.address()) {
        return Err(eyre!("Already validated {}", task_id));
    }

    let request = node.get_task_request(task_id).await?;

    // FIXME: will add validation prompt here
    let scores = (0..request.parameters.numGenerations)
        .map(|_| parse_ether("1.0").unwrap())
        .collect::<Vec<_>>();
    let metadata = Bytes::default();

    // FIXME: can add Arweave trick for metadata here

    // mine nonce
    let nonce = mine_nonce(
        request.parameters.difficulty,
        &request.requester,
        &node.address(),
        &request.input,
        &task_id,
    )
    .0;

    let tx_hash = node
        .respond_validation(task_id, scores, metadata, nonce)
        .await?;
    Ok(Some(tx_hash))
}
