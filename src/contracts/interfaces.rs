use alloy::sol;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ERC20,
    "./src/contracts/abi/ERC20.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    OracleRegistry,
    "./src/contracts/abi/LLMOracleRegistry.json"
);

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    OracleCoordinator,
    "./src/contracts/abi/LLMOracleCoordinator.json"
);
