use eyre::Result;

use super::PostProcess;

/// An identity post-processor that does nothing.
/// Input is directed to output and metadata is empty.
#[derive(Default)]
pub struct IdentityPostProcessor;

impl PostProcess for IdentityPostProcessor {
    const PROTOCOL: &'static str = "";

    fn post_process(&self, input: String) -> Result<(String, String)> {
        Ok((input, String::default()))
    }
}
