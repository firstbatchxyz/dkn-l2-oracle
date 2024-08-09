use crate::{compute, contracts::*, DriaOracleConfig};

use alloy::contract::EventPoller;
use alloy::primitives::Bytes;
use alloy::providers::fillers::{
    ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller,
};
use alloy::{
    network::{Ethereum, EthereumWallet},
    primitives::{Address, TxHash, U256},
    providers::{Identity, Provider, ProviderBuilder, RootProvider},
    transports::http::{Client, Http},
};
use alloy_chains::Chain;
use eyre::{eyre, Context, Result};
use futures_util::StreamExt;
use std::env;
use OracleCoordinator::{
    getResponsesReturn, getValidationsReturn, requestsReturn, StatusUpdate, TaskResponse,
};

#[cfg(test)]
use alloy::node_bindings::{Anvil, AnvilInstance};

// TODO: use a better type for these
type DriaOracleProviderTransport = Http<Client>;
type DriaOracleProvider = FillProvider<
    JoinFill<
        JoinFill<JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>, ChainIdFiller>,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<DriaOracleProviderTransport>,
    DriaOracleProviderTransport,
    Ethereum,
>;

pub struct DriaOracle {
    pub config: DriaOracleConfig,
    pub address: Address,
    pub contract_addresses: ContractAddresses,
    pub provider: DriaOracleProvider,
}

impl DriaOracle {
    /// Creates a new Anvil instance that forks the chain at the configured RPC URL.
    ///
    /// Return the node instance and the Anvil instance.
    /// Note that when Anvil instance is dropped, you will lose the forked chain.
    #[cfg(test)]
    pub async fn new_anvil(mut config: DriaOracleConfig) -> Result<(Self, AnvilInstance)> {
        let anvil = Anvil::new().fork(config.rpc_url.clone()).try_spawn()?;
        let anvil_rpc_url = anvil.endpoint_url();

        let config = config.with_rpc_url(anvil_rpc_url);
        let node = Self::new(config.to_owned()).await?;

        Ok((node, anvil))
    }

    /// Creates a new oracle node with the given private key and connected to the chain at the given RPC URL.
    ///
    /// The contract addresses are chosen based on the chain id returned from the provider.
    pub async fn new(config: DriaOracleConfig) -> Result<Self> {
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(config.wallet.clone())
            .on_http(config.rpc_url.clone());

        let chain_id_u64 = provider
            .get_chain_id()
            .await
            .wrap_err("Could not get chain id")?;
        let chain = Chain::from_id(chain_id_u64);
        let contract_addresses = ADDRESSES[&chain].clone();

        let node = Self {
            address: config.address.clone(),
            config,
            contract_addresses,
            provider,
        };

        node.check_contract_sizes().await?;
        node.check_contract_tokens().await?;

        Ok(node)
    }

    /// Returns the connected chain.
    pub async fn get_chain(&self) -> Result<Chain> {
        let chain_id_u64 = self
            .provider
            .get_chain_id()
            .await
            .wrap_err("Could not get chain id")?;

        Ok(Chain::from_id(chain_id_u64))
    }
    /// Returns ETH balance and configured token balance.
    pub async fn balances(&self) -> Result<[TokenBalance; 2]> {
        let token = ERC20::new(self.contract_addresses.token, self.provider.clone());
        let token_balance = token.balanceOf(self.address).call().await?._0;
        let token_symbol = token.symbol().call().await?._0;

        let eth_balance = self.provider.get_balance(self.address).await?;

        Ok([
            TokenBalance::new(eth_balance, "ETH".to_string(), None),
            TokenBalance::new(
                token_balance,
                token_symbol,
                Some(self.contract_addresses.token),
            ),
        ])
    }

    /// Request an oracle task. This is not done by the oracle normally, but we have it added for testing purposes.
    pub async fn request(
        &self,
        input: Bytes,
        models: Bytes,
        difficulty: u8,
        num_gens: u64,
        num_vals: u64,
    ) -> Result<TxHash> {
        let coordinator =
            OracleCoordinator::new(self.contract_addresses.coordinator, self.provider.clone());

        let req = coordinator.request(input, models, difficulty, num_gens, num_vals);
        let tx = req
            .send()
            .await
            .map_err(contract_error_report)
            .wrap_err("Could not request task.")?;

        log::info!("Hash: {:?}", tx.tx_hash());
        let awaited_tx_hash = tx.watch().await?;
        Ok(awaited_tx_hash)
    }

    /// Register the oracle with the registry.
    pub async fn register(&self, kind: OracleKind) -> Result<TxHash> {
        let registry = OracleRegistry::new(self.contract_addresses.registry, self.provider.clone());

        let req = registry.register(kind.into());
        let tx = req
            .send()
            .await
            .map_err(contract_error_report)
            .wrap_err(eyre!("Could not register."))?;

        log::info!("Hash: {:?}", tx.tx_hash());
        let awaited_tx_hash = tx.watch().await?;
        Ok(awaited_tx_hash)
    }

    /// Unregister from the oracle registry.
    pub async fn unregister(&self, kind: OracleKind) -> Result<TxHash> {
        let registry = OracleRegistry::new(self.contract_addresses.registry, self.provider.clone());

        let req = registry.unregister(kind.into());
        let tx = req
            .send()
            .await
            .map_err(contract_error_report)
            .wrap_err("Could not unregister.")?;

        log::info!("Hash: {:?}", tx.tx_hash());
        let awaited_tx_hash = tx.watch().await?;
        Ok(awaited_tx_hash)
    }

    pub async fn is_registered(&self, kind: OracleKind) -> Result<bool> {
        let registry = OracleRegistry::new(self.contract_addresses.registry, self.provider.clone());

        let is_registered = registry
            .isRegistered(self.address, kind.into())
            .call()
            .await?;
        Ok(is_registered._0)
    }

    pub async fn transfer_from(&self, from: Address, to: Address, amount: U256) -> Result<TxHash> {
        let token = ERC20::new(self.contract_addresses.token, self.provider.clone());

        let req = token.transferFrom(from, to, amount);
        let tx = req
            .send()
            .await
            .map_err(contract_error_report)
            .wrap_err("Could not transfer tokens.")?;

        log::info!("Hash: {:?}", tx.tx_hash());
        let awaited_tx_hash = tx.watch().await?;
        Ok(awaited_tx_hash)
    }

    pub async fn approve(&self, spender: Address, amount: U256) -> Result<TxHash> {
        let token = ERC20::new(self.contract_addresses.token, self.provider.clone());

        let req = token.approve(spender, amount);
        let tx = req
            .send()
            .await
            .map_err(contract_error_report)
            .wrap_err("Could not approve tokens.")?;

        log::info!("Hash: {:?}", tx.tx_hash());
        let awaited_tx_hash = tx.watch().await?;
        Ok(awaited_tx_hash)
    }

    pub async fn allowance(&self, owner: Address, spender: Address) -> Result<TokenBalance> {
        let token = ERC20::new(self.contract_addresses.token, self.provider.clone());
        let token_symbol = token.symbol().call().await?._0;

        let allowance = token.allowance(owner, spender).call().await?._0;
        Ok(TokenBalance::new(
            allowance,
            token_symbol,
            Some(self.contract_addresses.token),
        ))
    }

    /// Returns the amount of tokens to be staked to registry.
    pub async fn registry_stake_amount(&self, kind: OracleKind) -> Result<TokenBalance> {
        let registry = OracleRegistry::new(self.contract_addresses.registry, self.provider.clone());

        let stake_amount = registry.getStakeAmount(kind.into()).call().await?._0;

        // return the symbol as well
        let token_address = registry.token().call().await?._0;
        let token = ERC20::new(token_address, self.provider.clone());
        let token_symbol = token.symbol().call().await?._0;

        Ok(TokenBalance::new(
            stake_amount,
            token_symbol,
            Some(self.contract_addresses.token),
        ))
    }

    /// Returns the task request with the given id.
    pub async fn get_task_request(
        &self,
        task_id: U256,
    ) -> Result<OracleCoordinator::requestsReturn> {
        let coordinator =
            OracleCoordinator::new(self.contract_addresses.coordinator, self.provider.clone());

        let request = coordinator.requests(task_id).call().await?;
        Ok(request)
    }

    /// Returns the generation responses to a given task request.
    pub async fn get_task_responses(&self, task_id: U256) -> Result<Vec<TaskResponse>> {
        let coordinator =
            OracleCoordinator::new(self.contract_addresses.coordinator, self.provider.clone());

        let responses = coordinator.getResponses(task_id).call().await?;
        Ok(responses._0)
    }

    pub async fn respond_generation(
        &self,
        task_id: U256,
        response: Bytes,
        metadata: Bytes,
        nonce: U256,
    ) -> Result<TxHash> {
        let coordinator =
            OracleCoordinator::new(self.contract_addresses.coordinator, self.provider.clone());

        let req = coordinator.respond(task_id, nonce, response, metadata);
        let tx = req
            .send()
            .await
            .map_err(contract_error_report)
            .wrap_err("Could not respond to generation.")?;

        log::info!("Hash: {:?}", tx.tx_hash());
        let awaited_tx_hash = tx.watch().await?;
        Ok(awaited_tx_hash)
    }

    pub async fn respond_validation(
        &self,
        task_id: U256,
        scores: Vec<U256>,
        metadata: Bytes,
        nonce: U256,
    ) -> Result<TxHash> {
        let coordinator =
            OracleCoordinator::new(self.contract_addresses.coordinator, self.provider.clone());

        let req = coordinator.validate(task_id, nonce, scores, metadata);
        let tx = req
            .send()
            .await
            .map_err(contract_error_report)
            .wrap_err("Could not respond to validation.")?;

        log::info!("Hash: {:?}", tx.tx_hash());
        let awaited_tx_hash = tx.watch().await?;
        Ok(awaited_tx_hash)
    }

    /// Subscribes to events & processes tasks.
    pub async fn subscribe_to_tasks(
        &self,
    ) -> Result<EventPoller<DriaOracleProviderTransport, StatusUpdate>> {
        let coordinator =
            OracleCoordinator::new(self.contract_addresses.coordinator, self.provider.clone());

        Ok(coordinator.StatusUpdate_filter().watch().await?)
    }

    /// Checks contract sizes to ensure they are deployed.
    ///
    /// Returns an error if any of the contracts are not deployed.
    pub async fn check_contract_sizes(&self) -> Result<()> {
        let coordinator_size = self
            .provider
            .get_code_at(self.contract_addresses.coordinator)
            .await?;
        if coordinator_size.is_empty() {
            return Err(eyre!("Coordinator contract not deployed."));
        }
        let registry_size = self
            .provider
            .get_code_at(self.contract_addresses.registry)
            .await?;
        if registry_size.is_empty() {
            return Err(eyre!("Registry contract not deployed."));
        }
        let token_size = self
            .provider
            .get_code_at(self.contract_addresses.token)
            .await?;
        if token_size.is_empty() {
            return Err(eyre!("Token contract not deployed."));
        }

        log::debug!("Coordinator codesize: {}", coordinator_size);
        log::debug!("Registry codesize: {}", registry_size);
        log::debug!("Token codesize: {}", token_size);

        Ok(())
    }

    /// Ensures that the registry & coordinator tokens match the expected token.
    pub async fn check_contract_tokens(&self) -> Result<()> {
        let coordinator =
            OracleCoordinator::new(self.contract_addresses.coordinator, self.provider.clone());
        let registry = OracleRegistry::new(self.contract_addresses.registry, self.provider.clone());

        // check registry
        let registry_token = registry.token().call().await?._0;
        if registry_token != self.contract_addresses.token {
            return Err(eyre!("Registry token does not match."));
        }

        // check coordinator
        let coordinator_token = coordinator.feeToken().call().await?._0;
        if coordinator_token != self.contract_addresses.token {
            return Err(eyre!("Registry token does not match."));
        }

        Ok(())
    }

    pub async fn get_task(
        &self,
        task_id: U256,
    ) -> Result<(requestsReturn, getResponsesReturn, getValidationsReturn)> {
        let coordinator =
            OracleCoordinator::new(self.contract_addresses.coordinator, self.provider.clone());

        let request = coordinator.requests(task_id).call().await?;
        let responses = coordinator.getResponses(task_id).call().await?;
        let validations = coordinator.getValidations(task_id).call().await?;

        Ok((request, responses, validations))
    }
}

impl core::fmt::Display for DriaOracle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Dria Oracle Node v{}\nAddress: {}\nRPC URL: {}",
            env!("CARGO_PKG_VERSION"),
            self.address,
            self.config.rpc_url,
        )
    }
}
