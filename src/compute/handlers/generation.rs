use crate::{
    compute::GenerationRequest,
    contracts::{bytes32_to_string, bytes_to_string},
    data::Arweave,
    mine_nonce, DriaOracle,
};
use alloy::{
    primitives::{FixedBytes, U256},
    rpc::types::TransactionReceipt,
};
use dkn_workflows::DriaWorkflowsConfig;
use eyre::{Context, Result};

/// Handles a generation request.
///
/// 1. First, we check if we have already responded to the task.
///    Contract will revert even if we dont do this check ourselves, but its better to provide the error here.
///
/// 2. Then, we check if our models are compatible with the request. If not, we return an error.
pub async fn handle_generation(
    node: &DriaOracle,
    workflows: &DriaWorkflowsConfig,
    task_id: U256,
    protocol: FixedBytes<32>,
) -> Result<Option<TransactionReceipt>> {
    log::info!("Handling generation task {}", task_id);

    // check if we have responded to this generation already
    log::debug!("Checking existing generation responses");
    let responses = node.get_task_responses(task_id).await?;
    if responses.iter().any(|r| r.responder == node.address()) {
        log::debug!("Already responded to {} with generation", task_id);
        return Ok(None);
    }

    // fetch the request from contract
    log::debug!("Fetching the task request");
    let request = node
        .get_task_request(task_id)
        .await
        .wrap_err("could not get task")?;

    // choose model based on the request
    log::debug!("Choosing model to use");
    let models_string = bytes_to_string(&request.models)?;
    let models_vec = models_string.split(',').map(|s| s.to_string()).collect();
    let (_, model) = workflows.get_any_matching_model(models_vec)?;
    log::debug!("Using model: {} from {}", model, models_string);

    // parse protocol string early, in case it cannot be parsed
    let protocol_string = bytes32_to_string(&protocol)?;

    // execute task
    log::debug!("Executing the workflow");
    let mut input = GenerationRequest::try_parse_bytes(&request.input).await?;
    let output = input.execute(model, Some(node)).await?;
    log::debug!("Output: {}", output);

    // post-processing
    log::debug!(
        "Post-processing the output for protocol: {}",
        protocol_string
    );
    let (output, metadata, use_storage) = GenerationRequest::post_process(output, &protocol_string).await?;

    // uploading to storage
    log::debug!("Uploading output to storage");
    let arweave = Arweave::new_from_env()?;
    let output = if use_storage {
        arweave.put_if_large(output).await?
    } else {
        output
    };
    let metadata = arweave.put_if_large(metadata).await?;

    // mine nonce
    log::debug!("Mining nonce for task");
    let nonce = mine_nonce(
        request.parameters.difficulty,
        &request.requester,
        &node.address(),
        &request.input,
        &task_id,
    )
    .nonce;

    // respond
    log::debug!("Responding with generation");
    let tx_receipt = node
        .respond_generation(task_id, output, metadata, nonce)
        .await?;
    Ok(Some(tx_receipt))
}
