#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use eyre::{eyre, Context};
    use reqwest::Client;

    const BASE_URL: &str = "https://gateway.irys.xyz";
    const KEYPAIR_PATH: &str = "./secrets/testing.json";

    // example data: https://gateway.irys.xyz/Zg6CZYfxXCWYnCuKEpnZCYfy7ghit1_v4-BCe53iWuA
    #[tokio::test]
    async fn test_download_data() -> eyre::Result<()> {
        let key = "Zg6CZYfxXCWYnCuKEpnZCYfy7ghit1_v4-BCe53iWuA";
        let url = format!("{}/{}", BASE_URL, key); // TODO: key
        let client = Client::new();
        let response = client
            .get(&url)
            .send()
            .await
            .wrap_err("Failed to fetch from Arweave")?;

        if !response.status().is_success() {
            return Err(eyre!("Failed to fetch from Arweave: {}", response.status()));
        }

        let res = response.text().await?;

        println!("Response: {}", res);

        Ok(())
    }
}
