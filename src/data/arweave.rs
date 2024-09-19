use super::traits::OracleExternalData;
use async_trait::async_trait;
use bundlr_sdk::{currency::arweave::ArweaveBuilder, tags::Tag, BundlrBuilder};
use bytes::Bytes;
use eyre::{eyre, Context, Result};
use reqwest::{Client, Url};
use std::path::PathBuf;

/// An upload response.
///
/// ```js
/// {
///  block: 1513928,
///  deadlineHeight: 1513928,
///  id: '<base64-string>',
///  public: '<base64-string>',
///  signature: '<base64-string>',
///  timestamp: 1726753873618,
///  validatorSignatures: [],
///  version: '1.0.0'
///}
///```
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct UploadResponse {
    block: u64,
    deadline_height: u64,
    id: String,
    public: String,
    signature: String,
    timestamp: u64,
    validator_signatures: Vec<String>,
    version: String,
}

const DEFAULT_BASE_URL: &str = "https://node1.bundlr.network"; // "https://gateway.irys.xyz";

/// External data storage for Arweave.
///
/// - `put` corresponds to uploading (via Irys)
/// - `get` corresponds to downloading
pub struct Arweave {
    /// Path to Arweave keypair (usually JSON)
    wallet: PathBuf,
    /// Base URL for Arweave gateway, e.g:
    /// - https://gateway.irys.xyz
    /// - https://node1.bundlr.network
    base_url: Url,
    /// Reqwest client
    client: Client,
}

impl Arweave {
    /// Creates a new Arweave instance using the base URL.
    pub fn new(base_url: &str, wallet: &str) -> Result<Self> {
        let wallet = PathBuf::from(wallet);
        if !wallet.try_exists()? {
            return Err(eyre!("Wallet path does not exist."));
        }

        Ok(Self {
            wallet,
            base_url: Url::parse(base_url)?,
            client: Client::new(),
        })
    }

    async fn upload(&self, value: Bytes) -> Result<String> {
        // create tag
        let base_tag = Tag::new(
            "User-Agent",
            &format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION")),
        );

        // create Arweave currency instance
        let currency = ArweaveBuilder::new()
            .keypair_path(self.wallet.clone())
            .build()?;

        // create the Bundlr instance
        let bundlr = BundlrBuilder::new()
            .url(self.base_url.clone())
            .currency(currency)
            .fetch_pub_info()
            .await?
            .build()?;

        // create & sign transaction
        let mut tx = bundlr.create_transaction(value.into(), vec![base_tag])?;
        bundlr.sign_transaction(&mut tx).await?;
        let response_body = bundlr.send_transaction(tx).await?;
        let res = serde_json::from_value::<UploadResponse>(response_body)?;

        log::debug!("Uploaded to Arweave: {:#?}", res);
        Ok(res.id)
    }
}

impl Default for Arweave {
    fn default() -> Self {
        Self::new(DEFAULT_BASE_URL, "./secrets/wallet.json").unwrap()
    }
}

#[async_trait(?Send)]
impl OracleExternalData<String, Vec<u8>> for Arweave {
    async fn get(&self, key: String) -> Result<Vec<u8>> {
        let url = self.base_url.join(&key)?;
        let response = self
            .client
            .get(url)
            .send()
            .await
            .wrap_err("Failed to fetch from Arweave")?;

        if !response.status().is_success() {
            return Err(eyre!("Failed to fetch from Arweave: {}", response.status()));
        }

        let response_bytes = response.bytes().await?;
        Ok(response_bytes.into())
    }

    async fn put(&self, value: Vec<u8>) -> Result<String> {
        let value = Bytes::from(value);
        self.upload(value).await
    }

    /// Check if key is 64-characters and hex.
    fn is_key(key: String) -> bool {
        key.len() == 64 && key.chars().all(|c| c.is_ascii_hexdigit())
    }

    fn describe() -> String {
        "Arweave".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "run manually"]
    async fn test_download_data() -> Result<()> {
        // https://gateway.irys.xyz/Zg6CZYfxXCWYnCuKEpnZCYfy7ghit1_v4-BCe53iWuA
        let key = "AYamTrn2mXXwscwwpjAYEGPcNZxOu5rv5XHVng_sa0g";
        let arweave = Arweave::default();

        let result = arweave.get(key.to_string()).await?;
        println!("Result: {:?}", String::from_utf8_lossy(&result));
        let val = serde_json::from_slice::<String>(&result)?;
        assert_eq!(val, "Hello, Arweave!");

        Ok(())
    }

    #[tokio::test]
    #[ignore = "run manually with Arweave wallet"]
    async fn test_upload_and_download_data() -> Result<()> {
        let arweave = Arweave::default();
        let input_raw = "\"Hi there Im a test data\"";
        let input: String = serde_json::from_str(input_raw)?;

        let key = arweave.put(input.into()).await?;
        println!("Uploaded at \n{}", arweave.base_url.join(&key)?);

        let result = arweave.get(key.to_string()).await?;
        let val = serde_json::from_slice::<String>(&result)?;
        assert_eq!(val, input_raw);

        Ok(())
    }
}
