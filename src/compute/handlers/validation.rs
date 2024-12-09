use crate::{mine_nonce, DriaOracle};
use alloy::{
    primitives::{utils::parse_ether, Bytes, U256},
    rpc::types::TransactionReceipt,
};
use dkn_workflows::DriaWorkflowsConfig;
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

    // fetch request and generations
    log::debug!("Fetching the task request");
    let request = node
        .get_task_request(task_id)
        .await
        .wrap_err("could not get task request")?;

    // fetch each generation response & score it
    log::debug!("Starting validations");
    let mut scores = Vec::new();
    for i in 0..request.parameters.numGenerations {
        let score = parse_ether("1.0").unwrap();
        scores.push(score);
    }

    // FIXME: will add validation prompt here

    let scores = (0..request.parameters.numGenerations)
        .map(|_| parse_ether("1.0").unwrap())
        .collect::<Vec<_>>();
    // FIXME: metadata is empty for now, as dummy data
    // FIXME: can add Arweave trick for metadata here
    let metadata = Bytes::default();

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
