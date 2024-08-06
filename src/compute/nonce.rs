use alloy::core::dyn_abi::Encoder;
use alloy::primitives::{keccak256, Address, Bytes, U256};

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

    // target is 2^256 / 2^difficulty
    let target = U256::MAX >> difficulty;

    log::debug!("Being mining nonce for task {}", task_id);
    loop {
        // encode packed
        let mut encoder = Encoder::new();
        encoder.append_packed_seq(requester.as_slice());
        encoder.append_packed_seq(responder.as_slice());
        encoder.append_packed_seq(input.as_ref());
        encoder.append_packed_seq(task_id.as_le_slice());
        let encoded_packed = encoder.finish().into_iter().flatten().collect::<Vec<u8>>();

        // keccak256
        let digest = keccak256(encoded_packed);

        if U256::from_le_slice(digest.as_slice()) < target {
            return nonce;
        }

        nonce += big_one.clone();
    }
}
