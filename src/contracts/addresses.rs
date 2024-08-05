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
    pub(crate) weth: Address,
    pub(crate) registry: Address,
    pub(crate) coordinator: Address,
}

lazy_static! {
    /// Contract addresses per chain-id.
    pub static ref ADDRESSES: HashMap<Chain, ContractAddresses> = {
        let mut contracts = HashMap::new();

        // local
        contracts.insert(
            AnvilHardhat.into(),
            ContractAddresses {
                weth: address!("5FbDB2315678afecb367f032d93F642f64180aa3"),
                registry: address!("e7f1725E7734CE288F8367e1Bb143E90bb3F0512"),
                coordinator: address!("9fE46736679d2D9a65F0992F2272dE9f3c7fa6e0"),
            },
        );

        // base-sepolia
        contracts.insert(
            BaseSepolia.into(),
            ContractAddresses {
                weth: address!("4200000000000000000000000000000000000006"),
                registry: address!("0877022A137b8E8CE1C3020B9f047651dD02E37B"),
                coordinator: address!("EdC11Fe8a3fb4B8898c1ed988C5AB926BeA19B9C"),
            },
        );

        // TODO: add dria

        contracts
    };
}
