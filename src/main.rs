use dkn_oracle::DriaOracle;
use eyre::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set");

    let private_key_hex: String = env::var("SECRET_KEY").expect("SECRET_KEY must be set");
    let private_key_decoded = hex::decode(&private_key_hex).expect("should parse");
    let mut private_key = [0u8; 32];
    private_key.clone_from_slice(&private_key_decoded);

    let node = DriaOracle::new(&private_key, rpc_url).await;

    log::info!("{}", node);

    node.register().await;

    Ok(())
}
