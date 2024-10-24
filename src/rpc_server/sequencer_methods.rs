use crate::sequencer::sequencer::Sequencer;
use anyhow::{Context, Result};
use jsonrpsee::types::ErrorObject;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn register_sequencer_methods(
    module: &mut jsonrpsee::RpcModule<()>,
    sequencer: Arc<Mutex<Sequencer>>,
) -> Result<()> {
    let sequencer_clone = Arc::clone(&sequencer);

    module
        .register_async_method("get_pending_transactions", move |_params, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                let sequencer = sequencer.lock().await;
                match sequencer.get_pending_transaction_count().await {
                    Ok(count) => Ok(count),
                    Err(e) => {
                        let error = ErrorObject::owned(1, e.to_string(), None::<()>);
                        Err(error)
                    }
                }
            }
        })
        .context("Failed to register get_pending_transactions method")?;

    Ok(())
}
