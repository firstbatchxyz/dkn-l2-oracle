use async_trait::async_trait;
use eyre::Result;

/// A generalized external storage trait.
///
/// Putting a value should return a unique key, even for the same value uploaded multiple times.
/// Getting a value should be done with that returned key.
#[async_trait]
pub trait OracleExternalData {
    type Key;
    type Value;

    /// Returns the value (if exists) at the given key.
    async fn get(&self, key: Self::Key) -> Result<Self::Value>;

    /// Puts the value and returns the generated key.
    async fn put(&self, value: Self::Value) -> Self::Key;

    /// Checks if the key is valid.
    fn is_key(key: Self::Key) -> bool;

    /// Describes the implementation.
    fn describe() -> String;
}
