use crate::{
    compute::{execute::validate_generations, parse_downloadable},
    data::Arweave,
    mine_nonce, DriaOracle,
};
use alloy::{primitives::U256, rpc::types::TransactionReceipt};
use dkn_workflows::{DriaWorkflowsConfig, Model};
use eyre::{eyre, Context, Result};

/// Handles a validation request.
#[allow(unused)]
pub async fn handle_validation(
    node: &DriaOracle,
    workflows: &DriaWorkflowsConfig,
    task_id: U256,
) -> Result<Option<TransactionReceipt>> {
    log::info!("Handling validation task {}", task_id);

    // check if already responded as generator, because we cant validate our own answer
    log::debug!("Checking if we are a generator for this task");
    let responses = node.get_task_responses(task_id).await?;
    if responses.iter().any(|r| r.responder == node.address()) {
        log::debug!(
            "Cant validate {} with your own generation response",
            task_id
        );
        return Ok(None);
    }

    // check if we have validated anyways
    log::debug!("Checking if we have validated already");
    let validations = node.get_task_validations(task_id).await?;
    if validations.iter().any(|v| v.validator == node.address()) {
        return Err(eyre!("Already validated {}", task_id));
    }

    // fetch the request from contract
    log::debug!("Fetching the task request");
    let request = node
        .get_task_request(task_id)
        .await
        .wrap_err("could not get task request")?;

    // fetch each generation response & download its metadata
    log::debug!("Fetching response messages");
    let responses = node
        .get_task_responses(task_id)
        .await
        .wrap_err("could not get task responses")?;
    let mut generations = Vec::new();
    for response in responses {
        let metadata_str = parse_downloadable(&response.metadata).await?;
        generations.push(metadata_str);
    }
    let input = parse_downloadable(&request.input).await?;

    // validate each response
    // TODO: decide model w.r.t config
    log::debug!("Computing validation scores");
    let validations = validate_generations(input, generations, Model::GPT4o).await?;
    let scores = validations
        .iter()
        .map(|v| v.final_score_as_solidity_type())
        .collect::<Vec<_>>();
    let metadata =
        serde_json::to_string(&validations).wrap_err("could not serialize validations")?;

    // uploading to storage
    log::debug!("Uploading metadata to storage");
    let arweave = Arweave::new_from_env()?;
    let metadata = arweave.put_if_large(metadata.into()).await?;

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
    log::debug!("Responding with validation");
    let tx_receipt = node
        .respond_validation(task_id, scores, metadata, nonce)
        .await?;
    Ok(Some(tx_receipt))
}
