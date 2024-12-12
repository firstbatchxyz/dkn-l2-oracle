use alloy::{
    eips::BlockNumberOrTag,
    primitives::{address, utils::parse_ether, Address, U256},
    providers::ext::AnvilApi,
};
use dkn_workflows::{DriaWorkflowsConfig, Model};
use dria_oracle::{
    bytes_to_string, handle_request, string_to_bytes, DriaOracle, DriaOracleConfig, OracleKind,
    TaskStatus, WETH,
};
use eyre::Result;

const REGISTRY_OWNER: Address = address!("2aA47BC684aEC9093AB9E85c2CB19052887c1Aee");

#[tokio::test]
async fn test_oracle_string_input() -> Result<()> {
    dotenvy::dotenv().unwrap();

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Off)
        .filter_module("dria_oracle", log::LevelFilter::Debug)
        .filter_module("oracle_test", log::LevelFilter::Debug)
        .is_test(true)
        .try_init();

    // task setup
    let difficulty = 1;
    let models = string_to_bytes(Model::GPT4Turbo.to_string());
    let protocol = format!("test/{}", env!("CARGO_PKG_VERSION"));
    let input = string_to_bytes("What is the result of 2 + 2?".to_string());

    // node setup
    let workflows = DriaWorkflowsConfig::new(vec![Model::GPT4Turbo]);
    let config = DriaOracleConfig::new_from_env()?;
    let (node, _anvil) = DriaOracle::anvil_new(config).await?;

    // setup accounts
    let requester = node.connect(node.anvil_funded_wallet(None).await?);
    let generator = node.connect(node.anvil_funded_wallet(None).await?);
    let validator = node.connect(node.anvil_funded_wallet(None).await?);

    // buy some WETH for all people
    let amount = parse_ether("100").unwrap();
    for node in [&requester, &generator, &validator] {
        let token = WETH::new(node.addresses.token, &node.provider);
        let balance_before = node.get_token_balance(node.address()).await?;

        let call = token.deposit().value(amount);
        let _ = call.send().await?.get_receipt().await?;

        let balance_after = node.get_token_balance(node.address()).await?;
        assert!(balance_after.amount > balance_before.amount);
    }

    // whitelist validator with impersonation
    log::info!("Whitelisting validator");
    node.provider
        .anvil_impersonate_account(REGISTRY_OWNER)
        .await?;
    node.provider.anvil_mine(Some(U256::from(1)), None).await?;
    node.whitelist(validator.address()).await?;
    node.provider
        .anvil_stop_impersonating_account(REGISTRY_OWNER)
        .await?;

    assert!(node.is_whitelisted(validator.address()).await?);

    // register validator oracle
    validator.register(OracleKind::Validator).await?;
    assert!(validator.is_registered(OracleKind::Validator).await?);

    // register generator oracle
    generator.register(OracleKind::Generator).await?;
    assert!(generator.is_registered(OracleKind::Generator).await?);

    // approve some tokens for the coordinator from requester
    requester
        .approve(node.addresses.coordinator, amount)
        .await?;

    // make a request with just one generation and validation request
    let request_receipt = requester
        .request(input, models, difficulty, 1, 1, protocol)
        .await?;

    // handle generation by reading the latest event
    let tasks = node
        .get_tasks_in_range(
            request_receipt.block_number.unwrap(),
            BlockNumberOrTag::Latest,
        )
        .await?;
    assert!(tasks.len() == 1);
    let (event, _) = tasks.into_iter().next().unwrap();
    let task_id = event.taskId;
    assert_eq!(event.statusBefore, TaskStatus::None as u8);
    assert_eq!(event.statusAfter, TaskStatus::PendingGeneration as u8);
    let generation_receipt =
        handle_request(&generator, &[OracleKind::Generator], &workflows, event)
            .await?
            .unwrap();

    // handle validation by reading the latest event
    let tasks = node
        .get_tasks_in_range(
            generation_receipt.block_number.unwrap(),
            BlockNumberOrTag::Latest,
        )
        .await?;
    assert!(tasks.len() == 1);
    let (event, _) = tasks.into_iter().next().unwrap();
    assert_eq!(event.taskId, task_id);
    assert_eq!(event.statusBefore, TaskStatus::PendingGeneration as u8);
    assert_eq!(event.statusAfter, TaskStatus::PendingValidation as u8);
    let validation_receipt =
        handle_request(&validator, &[OracleKind::Validator], &workflows, event)
            .await?
            .unwrap();

    let tasks = node
        .get_tasks_in_range(
            validation_receipt.block_number.unwrap(),
            BlockNumberOrTag::Latest,
        )
        .await?;
    assert!(tasks.len() == 1);
    let (event, _) = tasks.into_iter().next().unwrap();
    assert_eq!(event.taskId, task_id);
    assert_eq!(event.statusBefore, TaskStatus::PendingValidation as u8);
    assert_eq!(event.statusAfter, TaskStatus::Completed as u8);

    // get responses
    let responses = node.get_task_responses(task_id).await?;
    assert_eq!(responses.len(), 1);
    let response = responses.into_iter().next().unwrap();
    let output_string = bytes_to_string(&response.output)?;
    assert!(output_string.contains("4"), "output must contain 4");
    assert!(!response.score.is_zero(), "score must be non-zero");
    log::debug!("Output: {}", output_string);

    Ok(())
}
