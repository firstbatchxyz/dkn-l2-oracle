use alloy::sol_types::SolValue;
use eyre::Result;
use std::str::FromStr;

use super::PostProcess;

/// Swan post-processor that seeks for lines between `<buy_list>` and `</buy_list>`.
/// and returns the intermediate strings as an array of strings.
///
/// The original input is kept as metadata.
pub struct SwanPostProcessor {
    /// Start marker to look for to start collecting assets.
    start_marker: &'static str,
    /// End marker to look for to stop collecting assets.
    end_marker: &'static str,
}

impl SwanPostProcessor {
    /// Create a new `SwanPostProcessor` with the given start and end markers.
    pub fn new(start_marker: &'static str, end_marker: &'static str) -> Self {
        Self {
            start_marker,
            end_marker,
        }
    }
}

impl PostProcess for SwanPostProcessor {
    const PROTOCOL: &'static str = "swan";

    fn post_process(&self, input: String) -> Result<(String, String)> {
        // we will cast strings to Address here
        use alloy::primitives::Address;

        // first, collect the buy lines
        let mut collecting = false;
        let mut shopping_list_lines = Vec::new();
        for line in input.lines() {
            if line.contains(self.start_marker) {
                // if we see the buy_list start marker, we can start collecting lines
                collecting = true;
            } else if line.contains(self.end_marker) {
                // if we see the buy list end marker, we can stop collecting lines
                break;
            } else if collecting {
                // if we are collecting, this must be a buy line
                shopping_list_lines.push(line);
            }
        }

        // then, do post processing on them to cast them to `Address`
        // TODO: handle error
        let addresses = shopping_list_lines
            .into_iter()
            .map(|line| Address::from_str(line).unwrap())
            .collect::<Vec<Address>>();

        // `abi.encode` the list of addresses to be decodable by contract
        let addresses_encoded = addresses.abi_encode();
        let output = hex::encode(addresses_encoded);

        Ok((output, input))
    }
}

// TODO: TESTS
