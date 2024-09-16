use std::str::FromStr;

use alloy::{dyn_abi::abi, sol_types::SolValue};

/// A `PostProcess` is a trait that defines a post-processing step for a workflow at application level.
/// It's input is the raw output from the LLM, and it splits it into an output and metadata.
/// The output is the main thing that is used within the contract, metadata is externally checked.
pub trait PostProcess {
    const PROTOCOL: &'static str;

    fn post_process(&self, input: String) -> (String, String);
}

/// An identity post-processor that does nothing.
/// Input is directed to output and metadata is empty.
pub struct IdentityPostProcessor;

impl PostProcess for IdentityPostProcessor {
    const PROTOCOL: &'static str = "";

    fn post_process(&self, input: String) -> (String, String) {
        (input, String::default())
    }
}

/// Swan post-processor that seeks for lines between `<buy_list>` and `</buy_list>`.
/// and returns the intermediate strings as an array of strings.
///
/// The original input is kept as metadata.
pub struct SwanPostProcessor;
impl PostProcess for SwanPostProcessor {
    const PROTOCOL: &'static str = "swan";

    fn post_process(&self, input: String) -> (String, String) {
        // we will cast strings to Address here
        use alloy::primitives::Address;

        // first, collect the buy lines
        let mut collecting = false;
        let mut buy_lines = Vec::new();
        for line in input.lines() {
            if line.contains("<buy_list>") {
                // if we see the buy_list start marker, we can start collecting lines
                collecting = true;
            } else if line.contains("</buy_list>") {
                // if we see the buy list end marker, we can stop collecting lines
                break;
            } else if collecting {
                // if we are collecting, this must be a buy line
                buy_lines.push(line);
            }
        }

        // then, do post processing on them to cast them to `Address`
        let addresses = buy_lines
            .into_iter()
            .map(|line| Address::from_str(line).unwrap())
            .collect::<Vec<Address>>();

        // `abi.encode` the list of addresses to be decodable by contract
        let addresses_encoded = addresses.abi_encode();
        let output = hex::encode(addresses_encoded);

        (output, input)
    }
}
