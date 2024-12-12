use alloy::{primitives::Bytes, sol_types::SolValue};
use eyre::Result;
use std::str::FromStr;

use super::PostProcess;

/// Swan post-processor that seeks for lines between `<shop_list>` and `</shop_list>`.
/// and returns the intermediate strings as an array of strings.
///
/// The original input is kept as metadata.
pub struct SwanPurchasePostProcessor {
    /// Start marker to look for to start collecting assets.
    start_marker: &'static str,
    /// End marker to look for to stop collecting assets.
    end_marker: &'static str,
}

impl SwanPurchasePostProcessor {
    /// Create a new `SwanPostProcessor` with the given start and end markers.
    pub fn new(start_marker: &'static str, end_marker: &'static str) -> Self {
        Self {
            start_marker,
            end_marker,
        }
    }
}

impl PostProcess for SwanPurchasePostProcessor {
    const PROTOCOL: &'static str = "swan-buyer-purchase";

    fn post_process(&self, input: String) -> Result<(Bytes, Bytes, bool)> {
        // we will cast strings to Address here
        use alloy::primitives::Address;

        // first, collect the buy lines
        let mut collecting = false;
        let mut shopping_list_lines = Vec::new();
        for line in input.lines() {
            if line.contains(self.start_marker) {
                // if we see the shop_list start marker, we can start collecting lines
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

        Ok((Bytes::from(addresses_encoded), Bytes::from(input), false))
    }
}

#[cfg(test)]
mod tests {
    use alloy::{
        hex::FromHex,
        primitives::{address, Address},
    };

    use super::*;

    #[test]
    fn test_swan_purchase_post_processor() {
        const INPUT: &str = r#"
some blabla here and there

<shop_list>
0x4200000000000000000000000000000000000001
0x4200000000000000000000000000000000000002
0x4200000000000000000000000000000000000003
0x4200000000000000000000000000000000000004
</shop_list>
    
some more blabla here
                "#;

        let post_processor = SwanPurchasePostProcessor::new("<shop_list>", "</shop_list>");

        let (output, metadata, _) = post_processor.post_process(INPUT.to_string()).unwrap();
        assert_eq!(
            metadata,
            Bytes::from(INPUT),
            "metadata must be the same as input"
        );

        // the output is abi encoded 4 addresses, it has 6 elements:
        // offset | length | addr1 | addr2 | addr3 | addr4
        //
        // offset: 2, since addr1 starts from that index
        // length: 4, since there are 4 addresses
        let expected_output = Bytes::from_hex("000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000040000000000000000000000004200000000000000000000000000000000000001000000000000000000000000420000000000000000000000000000000000000200000000000000000000000042000000000000000000000000000000000000030000000000000000000000004200000000000000000000000000000000000004").unwrap();
        assert_eq!(
            output, expected_output,
            "output must be the same as expected"
        );

        let addresses = <Vec<Address>>::abi_decode(&output, true).unwrap();
        assert_eq!(addresses.len(), 4, "must have listed addresses");
    }

    #[test]
    fn test_swan_post_processor_2() {
        const INPUT: &str = r#"
<shop_list>
0x36f55f830D6E628a78Fcb70F73f9D005BaF88eE3
0xAd75C9358799e830F0c23a4BB28dF4D2cCCc8846
0x26F5B12b67D5F006826824A73F58b88D6bdAA74B
0x671527de058BaD60C6151cA29d501C87439bCF62
0x66FC9dC1De3db773891753CD257359A26e876305
</shop_list>
"#;

        let post_processor = SwanPurchasePostProcessor::new("<shop_list>", "</shop_list>");

        let (output, _, _) = post_processor.post_process(INPUT.to_string()).unwrap();
        let addresses = <Vec<Address>>::abi_decode(&output, true).unwrap();
        let expected_addresses = vec![
            address!("36f55f830D6E628a78Fcb70F73f9D005BaF88eE3"),
            address!("Ad75C9358799e830F0c23a4BB28dF4D2cCCc8846"),
            address!("26F5B12b67D5F006826824A73F58b88D6bdAA74B"),
            address!("671527de058BaD60C6151cA29d501C87439bCF62"),
            address!("66FC9dC1De3db773891753CD257359A26e876305"),
        ];
        assert_eq!(addresses, expected_addresses, "must have listed addresses");
    }
}
