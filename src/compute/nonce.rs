use alloy::primitives::{keccak256, Address, Bytes, U256};
use alloy::sol_types::SolValue;

/// Mines a nonce for the oracle proof-of-work.
///
/// Returns (nonce, candidate, target).
pub fn mine_nonce(
    difficulty: u8,
    requester: &Address,
    responder: &Address,
    input: &Bytes,
    task_id: &U256,
) -> (U256, U256, U256) {
    let big_one = U256::from(1);
    let mut nonce = U256::ZERO;

    // target is (2^256-1) / 2^difficulty
    let target = U256::MAX >> difficulty;

    log::debug!("Mining nonce for task {}", task_id);
    loop {
        // encode packed
        let mut message = Vec::new();
        task_id.abi_encode_packed_to(&mut message);
        input.abi_encode_packed_to(&mut message);
        requester.abi_encode_packed_to(&mut message);
        responder.abi_encode_packed_to(&mut message);
        nonce.abi_encode_packed_to(&mut message);

        // check hash
        let digest = keccak256(message.clone());
        let candidate = U256::from_be_bytes(*digest); // big-endian!
        if candidate < target {
            return (nonce, candidate, target);
        }

        nonce += big_one;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::address;

    #[test]
    fn test_nonce() {
        let requester = address!("0877022A137b8E8CE1C3020B9f047651dD02E37B");
        let responder = address!("0877022A137b8E8CE1C3020B9f047651dD02E37B");
        let input = vec![0x01, 0x02, 0x03].into();
        let task_id = U256::from(0x1234);
        let difficulty = 2;

        let (nonce, _candidate, _target) =
            mine_nonce(difficulty, &requester, &responder, &input, &task_id);
        assert!(!nonce.is_zero());

        println!("Nonce: {}", nonce);
        println!("Target: {:x}", _target);
        println!("Candidate: {:x}", _candidate);
    }
}
