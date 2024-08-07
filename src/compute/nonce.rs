use alloy::primitives::{keccak256, Address, Bytes, U256};
use alloy::sol_types::SolValue;

/// Mines a nonce for the oracle proof-of-work.
pub fn mine_nonce(
    difficulty: u8,
    requester: Address,
    responder: Address,
    input: Bytes,
    task_id: U256,
) -> U256 {
    let big_one = U256::from(1);
    let mut nonce = U256::ZERO;

    // target is (2^256-1) / 2^difficulty
    let target = U256::MAX >> difficulty;

    log::debug!("Mining nonce for task {}", task_id);
    loop {
        // encode packed
        let mut message = Vec::new();
        requester.abi_encode_packed_to(&mut message);
        responder.abi_encode_packed_to(&mut message);
        input.abi_encode_packed_to(&mut message);
        task_id.abi_encode_packed_to(&mut message);

        let digest = keccak256(message);
        if U256::from_be_bytes(*digest) < target {
            return nonce;
        }

        nonce += big_one.clone();
    }
}
