use std::time::Duration;

use crate::{
    compute::handle_request,
    contracts::{bytes_to_string, string_to_bytes, OracleKind, TaskStatus},
    DriaOracle,
};
use alloy::{
    eips::BlockNumberOrTag,
    primitives::{utils::format_ether, U256},
};
use dkn_workflows::{DriaWorkflowsConfig, Model};
use eyre::{eyre, Context, Result};
use futures_util::StreamExt;

/// Runs the main loop of the oracle node.
pub async fn run_oracle(
    node: &DriaOracle,
    mut kinds: Vec<OracleKind>,
    models: Vec<Model>,
    from_block: impl Into<BlockNumberOrTag> + Clone,
) -> Result<()> {
    // if kinds are not provided, use the registrations as kinds
    if kinds.is_empty() {
        for kind in [OracleKind::Generator, OracleKind::Validator] {
            if node.is_registered(kind).await? {
                kinds.push(kind);
            }
        }

        if kinds.is_empty() {
            return Err(eyre!("You are not registered as any type of oracle."))?;
        }
    } else {
        // otherwise, make sure we are registered to required kinds
        for kind in &kinds {
            if !node.is_registered(*kind).await? {
                return Err(eyre!("You need to register as {} first.", kind))?;
            }
        }
    }

    // prepare model config
    let mut model_config = DriaWorkflowsConfig::new(models);
    if model_config.models.is_empty() {
        return Err(eyre!("No models provided."))?;
    }
    let ollama_config = model_config.ollama.clone();
    model_config = model_config.with_ollama_config(
        ollama_config
            .with_min_tps(5.0)
            .with_timeout(Duration::from_secs(150)),
    );
    model_config.check_services().await?;

    // check previous tasks
    if from_block.clone().into() != BlockNumberOrTag::Latest {
        log::info!(
            "Checking previous tasks from {} until now.",
            from_block.clone().into()
        );
        let prev_tasks = node
            .get_tasks_in_range(from_block.clone(), BlockNumberOrTag::Latest)
            .await?;
        for (event, log) in prev_tasks {
            let task_id = event.taskId;
            log::info!(
                "Previous task: {} ({} -> {})",
                task_id,
                TaskStatus::try_from(event.statusBefore).unwrap_or_default(),
                TaskStatus::try_from(event.statusAfter).unwrap_or_default()
            );
            log::debug!(
                "Handling task {} (tx: {})",
                task_id,
                log.transaction_hash.unwrap_or_default()
            );
            match handle_request(node, &kinds, &model_config, event).await {
                Ok(Some(receipt)) => {
                    log::info!(
                        "Task {} processed successfully. (tx: {})",
                        task_id,
                        receipt.transaction_hash
                    )
                }
                Ok(None) => {
                    log::info!("Task {} ignored.", task_id)
                }
                Err(e) => log::error!("Could not process task: {:?}", e),
            }
        }
    }

    // handle new tasks with subscription
    log::info!(
        "Subscribing to LLMOracleCoordinator ({}) as {}",
        node.addresses.coordinator,
        kinds
            .iter()
            .map(|kind| kind.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );
    let event_poller = node
        .subscribe_to_tasks()
        .await
        .wrap_err("could not subscribe to tasks")?;

    log::info!("Listening for events...");
    event_poller
        .into_stream()
        .for_each(|log| async {
            match log {
                Ok((event, log)) => {
                    let task_id = event.taskId;
                    log::debug!(
                        "Handling task {} (tx: {})",
                        task_id,
                        log.transaction_hash.unwrap_or_default()
                    );
                    match handle_request(node, &kinds, &model_config, event).await {
                        Ok(Some(receipt)) => {
                            log::info!(
                                "Task {} processed successfully. (tx: {})",
                                task_id,
                                receipt.transaction_hash
                            )
                        }
                        Ok(None) => {
                            log::info!("Task {} ignored.", task_id)
                        }
                        Err(e) => log::error!("Could not process task: {:?}", e),
                    }
                }
                Err(e) => log::error!("Could not handle event: {}", e),
            }
        })
        .await;

    Ok(())
}

pub async fn view_task_events(
    node: &DriaOracle,
    from_block: impl Into<BlockNumberOrTag> + Clone,
    to_block: impl Into<BlockNumberOrTag> + Clone,
) -> Result<()> {
    let from_block: BlockNumberOrTag = from_block.clone().into();
    let to_block: BlockNumberOrTag = to_block.clone().into();
    log::info!(
        "Viewing task ids & statuses between blocks: {} - {}",
        from_block
            .as_number()
            .map(|n| n.to_string())
            .unwrap_or(from_block.to_string()),
        to_block
            .as_number()
            .map(|n| n.to_string())
            .unwrap_or(to_block.to_string())
    );

    let task_events = node.get_tasks_in_range(from_block, to_block).await?;

    for (event, _) in task_events {
        log::info!(
            "Task: {} ({} -> {})",
            event.taskId,
            TaskStatus::try_from(event.statusBefore).unwrap_or_default(),
            TaskStatus::try_from(event.statusAfter).unwrap_or_default()
        );
    }

    Ok(())
}

pub async fn view_task(node: &DriaOracle, task_id: U256) -> Result<()> {
    log::info!("Viewing task {}.", task_id);
    let (request, responses, validations) = node.get_task(task_id).await?;

    log::info!(
        "Request Information:\nRequester: {}\nStatus:    {}\nInput:     {}\nModels:    {}",
        request.requester,
        TaskStatus::try_from(request.status)?,
        bytes_to_string(&request.input)?,
        bytes_to_string(&request.models)?
    );

    log::info!("Responses:");
    if responses._0.is_empty() {
        log::warn!("There are no responses yet.");
    } else {
        for (idx, response) in responses._0.iter().enumerate() {
            log::info!(
                "Response  #{}\nOutput:    {}\nMetadata:  {}\nGenerator: {}",
                idx,
                bytes_to_string(&response.output)?,
                bytes_to_string(&response.metadata)?,
                response.responder
            );
        }
    }

    log::info!("Validations:");
    if validations._0.is_empty() {
        log::warn!("There are no validations yet.");
    } else {
        for (idx, validation) in validations._0.iter().enumerate() {
            log::info!(
                "Validation #{}\nScores:     {:?}\nMetadata:   {}\nValidator:  {}",
                idx,
                validation.scores,
                bytes_to_string(&validation.metadata)?,
                validation.validator
            );
        }
    }

    Ok(())
}

pub async fn request_task(
    node: &DriaOracle,
    input: &str,
    models: Vec<Model>,
    difficulty: u8,
    num_gens: u64,
    num_vals: u64,
    protocol: String,
) -> Result<()> {
    let input = string_to_bytes(input.to_string());
    let models_str = models
        .iter()
        .map(|m| m.to_string())
        .collect::<Vec<String>>()
        .join(",");
    let models = string_to_bytes(models_str);
    log::info!("Requesting a new task.");

    // get total fee for the request
    let total_fee = node
        .get_request_fee(difficulty, num_gens, num_vals)
        .await?
        .totalFee;

    // check balance
    let balance = node.get_token_balance(node.address()).await?.amount;
    if balance < total_fee {
        return Err(eyre!("Insufficient balance. Please fund your wallet."));
    }

    // check current allowance
    let allowance = node
        .allowance(node.address(), node.addresses.coordinator)
        .await?
        .amount;

    // make sure we have enough allowance
    if allowance < total_fee {
        let approval_amount = total_fee - allowance;
        log::info!(
            "Insufficient allowance. Approving the required amount: {}.",
            format_ether(approval_amount)
        );

        node.approve(node.addresses.coordinator, approval_amount)
            .await?;
        log::info!("Token approval successful.");
    }

    // make the request
    let receipt = node
        .request(input, models, difficulty, num_gens, num_vals, protocol)
        .await?;
    log::info!(
        "Task requested successfully. tx: {}",
        receipt.transaction_hash
    );

    Ok(())
}
