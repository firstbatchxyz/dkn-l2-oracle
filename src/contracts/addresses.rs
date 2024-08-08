use alloy::primitives::{address, Address};
use alloy_chains::{
    Chain,
    NamedChain::{AnvilHardhat, BaseSepolia},
};
use lazy_static::lazy_static;
use std::collections::HashMap;

/// Contract addresses.
#[derive(Debug, Clone)]
pub struct ContractAddresses {
    /// Token used within the registry and coordinator.
    pub(crate) token: Address,
    /// Oracle registry.
    pub(crate) registry: Address,
    /// Oracle coordinator.
    pub(crate) coordinator: Address,
}

impl std::fmt::Display for ContractAddresses {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Contract Addresses:\n  Token: {}\n  Registry: {}\n  Coordinator: {}",
            self.token, self.registry, self.coordinator
        )
    }
}

lazy_static! {
    /// Contract addresses per chain-id.
    pub static ref ADDRESSES: HashMap<Chain, ContractAddresses> = {
        let mut contracts = HashMap::new();

        // localhost
        contracts.insert(
            AnvilHardhat.into(),
            ContractAddresses {
                token: address!("5FbDB2315678afecb367f032d93F642f64180aa3"),
                registry: address!("e7f1725E7734CE288F8367e1Bb143E90bb3F0512"),
                coordinator: address!("9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0"),
            },
        );

        // base-sepolia
        contracts.insert(
            BaseSepolia.into(),
            ContractAddresses {
                token: address!("4200000000000000000000000000000000000006"),
                registry: address!("8D09081F1620A70e9102c3E331EF3749A56EA5F0"),
                coordinator: address!("303c4aD9DA1756f34a2CD1d20926B48780A3b9DE"),
            },
        );

        // TODO: add dria

        contracts
    };
}
