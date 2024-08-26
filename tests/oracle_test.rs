// use alloy::{network::EthereumWallet, providers::{ext::AnvilApi, WalletProvider}, signers::local::PrivateKeySigner};
// use dkn_oracle::{DriaOracle, DriaOracleConfig};
// use eyre::Result;

// #[tokio::test]
// async fn test_oracle_request() -> Result<()> {
//     let config = DriaOracleConfig::new_from_env()?;
//     let (node, anvil) = DriaOracle::new_anvil(config).await?;

//     // setup requester
//     let requester: PrivateKeySigner = anvil.keys()[1].clone().into();
//     let requester_wallet = EthereumWallet::from(requester);
//     let mut requester_provider = node.provider.clone();
//     *requester_provider.wallet_mut() = requester_wallet;

//     // Register oracle
//     let oracle_name = "TestOracle";
//     let _ = registry
//         .register_oracle(oracle.address(), oracle_name.into())
//         .send()
//         .await?;

//     // Set up requester
//     let requester: PrivateKeySigner = anvil.keys()[2].clone().into();
//     let requester_wallet = EthereumWallet::from(requester);
//     let mut requester_provider = provider.clone();
//     *requester_provider.wallet_mut() = requester_wallet;

//     // Create a request
//     let request_data = "Test request data".as_bytes();
//     let request_id = keccak256(request_data);
//     let _ = coordinator
//         .create_request(
//             oracle.address(),
//             Bytes::from(request_data.to_vec()),
//             U256::from(100000),
//         )
//         .send()
//         .await?;

//     // Verify the request was created
//     let stored_request = coordinator.get_request(request_id).call().await?;
//     assert_eq!(stored_request.requester, requester.address());
//     assert_eq!(stored_request.oracle, oracle.address());

//     Ok(())
// }
