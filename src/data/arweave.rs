use super::traits::OracleExternalData;
use async_trait::async_trait;
use eyre::{eyre, Context, Result};
use reqwest::{Client, Url};

const DEFAULT_BASE_URL: &str = "https://gateway.irys.xyz";

/// External data storage for Arweave.
///
/// - `put` corresponds to uploading
/// - `get` corresponds to downloading
pub struct Arweave {
    // keypair_path: PathBuf,
    base_url: Url,
    client: Client,
}

impl Arweave {
    /// Creates a new Arweave instance using the base URL.
    ///
    /// Base URL is most likely: https://gateway.irys.xyz
    pub fn new(base_url: &str) -> Result<Self> {
        // let keypair_path = PathBuf::from(keypair_path);
        // if !keypair_path.try_exists()? {
        //     return Err(eyre!("Keypair does not exist."));
        // }

        Ok(Self {
            base_url: Url::parse(base_url)?,
            client: Client::new(),
        })
    }
}

impl Default for Arweave {
    fn default() -> Self {
        Self::new(DEFAULT_BASE_URL).unwrap()
    }
}

#[async_trait]
impl OracleExternalData for Arweave {
    type Key = String;
    type Value = bytes::Bytes;

    async fn get(&self, key: Self::Key) -> Result<Self::Value> {
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
        Ok(response_bytes)
    }

    async fn put(&self, _: Self::Value) -> Self::Key {
        // TODO: implement Arweave client
        unimplemented!("not implemented")
    }

    /// Check if key is 64-characters and hex.
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
    async fn test_download_data() -> Result<()> {
        // https://gateway.irys.xyz/Zg6CZYfxXCWYnCuKEpnZCYfy7ghit1_v4-BCe53iWuA
        let key = "Zg6CZYfxXCWYnCuKEpnZCYfy7ghit1_v4-BCe53iWuA";
        let arweave = Arweave::default();

        let result = arweave.get(key.to_string()).await?;
        let val = serde_json::from_slice::<String>(&result)?;
        assert_eq!(val, "Hello, Arweave!");

        Ok(())
    }
}
