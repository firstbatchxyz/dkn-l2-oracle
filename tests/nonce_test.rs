use alloy::{
    network::EthereumWallet,
    node_bindings::Anvil,
    primitives::{address, Address, Bytes, U256},
    providers::ProviderBuilder,
    signers::local::PrivateKeySigner,
    sol,
};
use dkn_oracle::mine_nonce;
use eyre::Result;

// Codegen from embedded Solidity code and precompiled bytecode.
sol! {
    #[allow(missing_docs)]
    // solc v0.8.26; solc tests/contracts/TestNonce.sol --via-ir --optimize --bin
    #[sol(rpc, bytecode="60808060405234601557610194908161001a8239f35b5f80fdfe6080600436101561000e575f80fd5b5f3560e01c63f8c4172414610021575f80fd5b3461015a5760a036600319011261015a576024359067ffffffffffffffff821161015a573660238301121561015a57816004013567ffffffffffffffff811161015a57366024828501011161015a576044356001600160a01b038116810361015a576084359160ff831680930361015a57604091818592602460208501986004358a5201858501378201906bffffffffffffffffffffffff199060601b16838201523360601b6054820152606435606882015203016028810183526067601f1991011682019282841067ffffffffffffffff8511176101465760a0928492836040525f19901c908051832090608085525180938160808701528686015e5f84840186015281811115602085015260408401526060830152601f01601f19168101030190f35b634e487b7160e01b5f52604160045260245ffd5b5f80fdfea2646970667358221220d949f7ac783f721654995d06392a71caf3863f802400aa5dd084f7a132235f0064736f6c634300081a0033")]
    contract TestNonce {
        function assertValidNonce(uint256 taskId, bytes calldata input, address requester, uint256 nonce, uint8 difficulty) external
        view
        returns (bytes memory message, bool result, bytes32 candidate, uint256 target)
        {
            message = abi.encodePacked(taskId, input, requester, msg.sender, nonce);
            target = type(uint256).max >> uint256(difficulty);
            candidate = keccak256(message);
            result = uint256(candidate) <= target;
        }
    }
}

#[tokio::test]
async fn test_nonce() -> Result<()> {
    let anvil = Anvil::new().try_spawn()?;
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer);
    let rpc_url = anvil.endpoint().parse()?;
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(rpc_url);
    let contract = TestNonce::deploy(&provider).await?;

    // prepare parameters
    let difficulty = 2u8;
    let task_id = U256::from(1);
    let requester = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");
    let responder = Address::ZERO; // TODO: Anvil sets signer address to zero for some reason
    let input = Bytes::from_iter("im some bytes yallllll".bytes());

    // call contract
    let (nonce, _candidate, _target) =
        mine_nonce(difficulty, &requester, &responder, &input, &task_id);
    // println!("Nonce:     {}", nonce);
    // println!("Target:    {:x}", target);
    // println!("Candidate: {:x}", candidate);
    let contract_bytes = contract
        .assertValidNonce(task_id, input, requester, nonce, difficulty)
        .call()
        .await?;

    // println!("\nResult:    {}", contract_bytes.result);
    // println!("Target:    {:x}", contract_bytes.target);
    // println!("Candidate: {:x}", contract_bytes.candidate);
    // println!("Message:\n{:x}", contract_bytes.message);
    assert_eq!(contract_bytes.result, true);
    Ok(())
}
