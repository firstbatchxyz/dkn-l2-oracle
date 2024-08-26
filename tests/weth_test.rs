use alloy::{
    primitives::{address, utils::parse_ether, Address},
    sol,
};
use dkn_oracle::{DriaOracle, DriaOracleConfig};
use eyre::Result;

// Base WETH
sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    WETH,
    "./src/contracts/abi/IWETH9.json"
);

const WETH_ADDR: Address = address!("4200000000000000000000000000000000000006");

/// Using the forked blockchain, creates two accounts (alice, bob) and then,
///
/// 1. Alice buys WETH
/// 2. Alice approves Bob
/// 3. Bob transfers WETH from Alice
#[tokio::test]
async fn test_weth_transfer() -> Result<()> {
    let config = DriaOracleConfig::new_from_env()?;
    let (node, _anvil) = DriaOracle::anvil_new(config).await?;

    let init_balance = parse_ether("999")?;

    // setup alice
    let alice = node.connect(node.anvil_funded_wallet(Some(init_balance)).await?);
    let alice_token = WETH::new(WETH_ADDR, &alice.provider);

    // setup bob
    let bob = node.connect(node.anvil_funded_wallet(Some(init_balance)).await?);
    let bob_token = WETH::new(WETH_ADDR, &bob.provider);

    // set balances
    let alice_balance = node.get_native_balance(alice.address()).await?;
    let bob_balance = node.get_native_balance(bob.address()).await?;
    assert_eq!(alice_balance.amount, init_balance);
    assert_eq!(bob_balance.amount, init_balance);

    let alice_balance_before = node.get_token_balance(alice.address()).await?;
    let bob_balance_before = node.get_token_balance(bob.address()).await?;
    let amount = parse_ether("100")?;

    // alice buys WETH
    let _ = alice_token.deposit().value(amount).send().await?;
    let alice_balance_after = node.get_token_balance(alice.address()).await?;
    assert_eq!(
        alice_balance_after.amount - alice_balance_before.amount,
        amount
    );

    // alice approves bob
    let _ = alice_token.approve(bob.address(), amount).send().await?;

    // bob transfers WETH from alice
    let _ = bob_token
        .transferFrom(alice.address(), bob.address(), amount)
        .send()
        .await?;
    let alice_balance_after = node.get_token_balance(alice.address()).await?;
    let bob_balance_after = node.get_token_balance(bob.address()).await?;
    assert_eq!(alice_balance_after.amount, alice_balance_before.amount);
    assert_eq!(bob_balance_after.amount - bob_balance_before.amount, amount);

    Ok(())
}
