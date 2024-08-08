use crate::{contracts::OracleKind, DriaOracle};
use eyre::{eyre, Result};

// TODO: add cancellation here
/// Runs the main loop of the oracle node.
pub async fn run_oracle(node: &DriaOracle, kinds: Vec<OracleKind>) -> Result<()> {
    // TODO: specify oracle kinds to run here & check registry for that
    // make sure we are registered
    // if !node.is_registered().await? {
    //     return Err(eyre!("You need to register first."))?;
    // }

    node.process_tasks().await?;

    Ok(())
}
