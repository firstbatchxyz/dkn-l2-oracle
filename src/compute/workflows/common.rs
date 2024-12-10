use alloy::primitives::Bytes;
use eyre::{Context, Result};

use crate::{
    bytes_to_string,
    data::{Arweave, OracleExternalStorage},
};

/// Parses a given bytes input to a string, and if it is a storage key identifier it automatically
/// downloads the data from Arweave.
pub async fn parse_downloadable(input_bytes: &Bytes) -> Result<String> {
    // first, convert to string
    let mut input_string = bytes_to_string(input_bytes)?;

    // then, check storage
    if Arweave::is_key(input_string.clone()) {
        // if its a txid, we download the data and parse it again
        let input_bytes_from_arweave = Arweave::default()
            .get(input_string)
            .await
            .wrap_err("could not download from Arweave")?;

        // convert the input to string
        input_string = bytes_to_string(&input_bytes_from_arweave)?;
    }

    Ok(input_string)
}
