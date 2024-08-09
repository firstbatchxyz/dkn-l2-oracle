use async_trait::async_trait;
use eyre::Result;

/// A generalized external storage trait.
///
/// For Arweave, `put` corresponds to uploading and `get` corresponds to downloading.
#[async_trait]
pub trait OracleExternalData {
    type Key;
    type Value;

    /// Returns the value (if exists) at the given key.
    async fn get(&self, key: Self::Key) -> Result<Self::Value>;

    /// Puts the value and returns the generated key.
    async fn put(&self, value: Self::Value) -> Self::Key;

    /// Checks if the key is valid.
    fn is_key(&self, key: Self::Key) -> bool;

    /// Describes the implementation.
    fn describe() -> String;
}
