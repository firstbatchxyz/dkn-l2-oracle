use super::traits::OracleExternalStorage;
use alloy::primitives::Bytes;
use async_trait::async_trait;
use base64::prelude::*;
use bundlr_sdk::{currency::arweave::ArweaveBuilder, tags::Tag, BundlrBuilder};
use eyre::{eyre, Context, Result};
use reqwest::{Client, Url};
use std::{env, path::PathBuf};

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
#[allow(unused)]
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
const DEFAULT_WALLET_PATH: &str = "./secrets/wallet.json";
const DEFAULT_BYTE_LIMIT: usize = 1024; // 1KB

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
    /// Reqwest client.
    client: Client,
    /// Byte limit for the data to be considered for Arweave.
    ///
    /// - If the data exceeds this limit, it will be uploaded to Arweave.
    /// - Otherwise, it will be stored as is.
    byte_limit: usize,
}

impl Arweave {
    /// Creates a new Arweave instance.
    pub fn new(base_url: &str, wallet: &str, byte_limit: usize) -> Result<Self> {
        Ok(Self {
            wallet: PathBuf::from(wallet),
            base_url: Url::parse(base_url).wrap_err("could not parse base URL")?,
            client: Client::new(),
            byte_limit,
        })
    }

    /// Creates a new Arweave instance from the environment variables.
    ///
    /// Required environment variables:
    ///
    /// - `ARWEAVE_WALLET_PATH`
    /// - `ARWEAVE_BASE_URL`
    /// - `ARWEAVE_BYTE_LIMIT`
    ///
    /// All these variables have defaults if they are missing.
    pub fn new_from_env() -> Result<Self> {
        let wallet = env::var("ARWEAVE_WALLET_PATH").unwrap_or(DEFAULT_WALLET_PATH.to_string());
        let base_url = env::var("ARWEAVE_BASE_URL").unwrap_or(DEFAULT_BASE_URL.to_string());
        let byte_limit = env::var("ARWEAVE_BYTE_LIMIT")
            .unwrap_or(DEFAULT_BYTE_LIMIT.to_string())
            .parse::<usize>()
            .wrap_err("could not parse ARWEAVE_BYTE_LIMIT")?;

        Self::new(&base_url, &wallet, byte_limit)
    }

    /// Converts a base64 key string to hexadecimal string.
    ///
    /// Arweave uses base64 for keys but they are 32 bytes,
    /// we want to use hexadecimals to read them easily on-chain.
    #[inline(always)]
    pub fn base64_to_hex(key: &str) -> Result<String> {
        let decoded_key = BASE64_URL_SAFE_NO_PAD
            .decode(key.as_bytes())
            .wrap_err("could not decode base64 url")?;
        Ok(hex::encode(decoded_key))
    }

    /// Converts a hexadecimal key string to base64 string.
    ///
    /// Arweave uses base64 for keys but they are 32 bytes,
    /// we want to use hexadecimals to read them easily on-chain.
    #[inline(always)]
    pub fn hex_to_base64(key: &str) -> Result<String> {
        let decoded_key = hex::decode(key).wrap_err("could not decode hexadecimals")?;
        Ok(BASE64_URL_SAFE_NO_PAD.encode(&decoded_key))
    }

    /// Puts the value if it is larger than the byte limit.
    #[inline]
    pub async fn put_if_large(&self, value: Bytes) -> Result<Bytes> {
        let value_size = value.len();
        if value_size > self.byte_limit {
            log::info!(
                "Uploading large ({}B > {}B) value to Arweave",
                value_size,
                self.byte_limit
            );
            let key = self.put(value.clone()).await?;
            Ok(key.into())
        } else {
            Ok(value)
        }
    }
}

impl Default for Arweave {
    fn default() -> Self {
        Self::new(DEFAULT_BASE_URL, DEFAULT_WALLET_PATH, DEFAULT_BYTE_LIMIT)
            .expect("Failed to create Default Arweave instance")
    }
}

#[async_trait(?Send)]
impl OracleExternalStorage for Arweave {
    type Key = String;
    type Value = Bytes;

    async fn get(&self, key: Self::Key) -> Result<Self::Value> {
        if !Self::is_key(key.clone()) {
            return Err(eyre!("Invalid key for Arweave"));
        }
        let b64_key = Self::hex_to_base64(key.as_str())?;

        let url = self.base_url.join(&b64_key)?;
        log::debug!("Fetching from Arweave: {}", url);
        let response = self
            .client
            .get(url)
            .send()
            .await
            .wrap_err("failed to fetch from Arweave")?;

        if !response.status().is_success() {
            return Err(eyre!("Failed to fetch from Arweave: {}", response.status()));
        }

        let response_bytes = response.bytes().await?;
        Ok(response_bytes.into())
    }

    async fn put(&self, value: Self::Value) -> Result<Self::Key> {
        // ensure that wallet exists
        // NOTE: we do this here instead of `new` so that we can work without any wallet
        // in case we only want to download data.
        if !self.wallet.try_exists()? {
            return Err(eyre!("Wallet does not exist at {}.", self.wallet.display()));
        }

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
        log::info!("Uploaded at {}", self.base_url.join(&res.id)?);

        // the key is in base64 format, we want to convert that to hexadecimals
        let key = Self::base64_to_hex(&res.id)?;
        Ok(key)
    }

    /// Check if key is 64-characters and hex.
    #[inline(always)]
    fn is_key(key: Self::Key) -> bool {
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
        let key = Arweave::base64_to_hex("Zg6CZYfxXCWYnCuKEpnZCYfy7ghit1_v4-BCe53iWuA")?;
        let arweave = Arweave::default();

        let result = arweave.get(key.to_string()).await?;
        let val = serde_json::from_slice::<String>(&result)?;
        assert_eq!(val, "Hello, Arweave!");

        Ok(())
    }

    #[tokio::test]
    #[ignore = "run manually with Arweave wallet"]
    async fn test_upload_and_download_data() -> Result<()> {
        let arweave = Arweave::default();
        let input = b"Hi there Im a test data".to_vec();

        // put data
        let key = arweave.put(input.clone().into()).await?;
        // println!("Uploaded at \n{}", arweave.base_url.join(&key)?);

        // get it again
        let result = arweave.get(key.to_string()).await?;
        assert_eq!(input, result);

        Ok(())
    }

    #[test]
    fn test_key_conversions() -> Result<()> {
        let key_b64 = "Zg6CZYfxXCWYnCuKEpnZCYfy7ghit1_v4-BCe53iWuA";

        let key_hex = Arweave::base64_to_hex(key_b64)?;
        assert!(Arweave::is_key(key_hex.clone()));

        let key_b64_again = Arweave::hex_to_base64(&key_hex)?;
        assert_eq!(key_b64, key_b64_again);

        Ok(())
    }
}
