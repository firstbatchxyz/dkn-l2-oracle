use crate::{
    compute::{handle_request, ModelConfig},
    contracts::{bytes_to_string, OracleKind, TaskStatus},
    DriaOracle,
};
use alloy::{eips::BlockNumberOrTag, primitives::U256};
use eyre::{eyre, Context, Result};
use futures_util::StreamExt;
use ollama_workflows::Model;

// TODO: add cancellation here
/// Runs the main loop of the oracle node.
pub async fn run_oracle(
    node: &DriaOracle,
    kinds: Vec<OracleKind>,
    models: Vec<Model>,
    from_block: impl Into<BlockNumberOrTag> + Clone,
) -> Result<()> {
    // make sure we are registered to required kinds
    for kind in &kinds {
        if !node.is_registered(*kind).await? {
            return Err(eyre!("You need to register as {} first.", kind))?;
        }
    }

    // prepare model config
    let model_config = ModelConfig::new(models);
    if model_config.models_providers.is_empty() {
        return Err(eyre!("No models provided."))?;
    }
    model_config.check_providers().await?;

    // check previous tasks
    if from_block.clone().into() != BlockNumberOrTag::Latest {
        log::info!(
            "Checking previous tasks from {} until now.",
            from_block.clone().into()
        );
        let prev_tasks = node
            .get_tasks(from_block.clone(), BlockNumberOrTag::Latest)
            .await?;
        for (event, log) in prev_tasks {
            let task_id = event.taskId;
            log::info!(
                "Previous task: {} ({} -> {})",
                task_id,
                TaskStatus::try_from(event.statusBefore).unwrap_or_default(),
                TaskStatus::try_from(event.statusAfter).unwrap_or_default()
            );
            match handle_request(node, &kinds, &model_config, event, log).await {
                Ok(Some(tx_hash)) => {
                    log::info!("Task {} processed successfully. (tx: {})", task_id, tx_hash)
                }
                Ok(None) => {
                    log::info!("Task {} ignored.", task_id)
                }
                Err(e) => log::error!("Could not process task: {}", e),
            }
        }
    }

    // handle new tasks with subscription
    let event_poller = node
        .subscribe_to_tasks()
        .await
        .wrap_err("Could not subscribe")?;
    log::info!(
        "Subscribed to LLMOracleCoordinator ({}) as {}",
        node.contract_addresses.coordinator,
        kinds
            .iter()
            .map(|kind| kind.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    event_poller
        .into_stream()
        .for_each(|log| async {
            match log {
                Ok((event, log)) => {
                    let task_id = event.taskId;
                    match handle_request(node, &kinds, &model_config, event, log).await {
                        Ok(Some(tx_hash)) => {
                            log::info!("Task {} processed successfully. (tx: {})", task_id, tx_hash)
                        }
                        Ok(None) => {
                            log::info!("Task {} ignored.", task_id)
                        }
                        Err(e) => log::error!("Could not process task: {:#?}", e),
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
    log::info!(
        "Viewing task ids & statuses between blocks: {} - {}",
        from_block.clone().into(),
        to_block.clone().into()
    );

    let task_events = node.get_tasks(from_block, to_block).await?;

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

    log::info!("Request Information:");
    log::info!("Requester: {}", request.requester);
    log::info!("Status:    {}", TaskStatus::try_from(request.status)?);
    log::info!("Input:     {}", bytes_to_string(&request.input)?);
    log::info!("Models:    {}", bytes_to_string(&request.models)?);

    log::info!("Responses:");
    if responses._0.is_empty() {
        log::warn!("There are no responses yet.");
    } else {
        for (idx, response) in responses._0.iter().enumerate() {
            log::info!("Response  #{}", idx);
            log::info!("Output:    {}", bytes_to_string(&response.output)?);
            log::info!("Metadata:  {}", bytes_to_string(&response.metadata)?);
            log::info!("Generator: {}", response.responder);
        }
    }

    log::info!("Validations:");
    if validations._0.is_empty() {
        log::warn!("There are no validations yet.");
    } else {
        for (idx, validation) in validations._0.iter().enumerate() {
            log::info!("Validation #{}", idx);
            log::info!("Scores:     {:?}", validation.scores);
            log::info!("Metadata:   {}", bytes_to_string(&validation.metadata)?);
            log::info!("Validator:  {}", validation.validator);
        }
    }

    Ok(())
}
