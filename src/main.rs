mod mempool;
mod sequencer;
mod rpc_server;
mod transaction;

use anyhow::Result; 
use std::sync::Arc;
use crate::mempool::mempool::Mempool;
use crate::sequencer::sequencer::Sequencer;
use crate::rpc_server::rpc_server::start_rpc_server;
use tokio::sync::Mutex; 

#[tokio::main]
async fn main() -> Result<()> {
    let mempool = Mempool::new("mempool_db");
    let sequencer = Arc::new(Mutex::new(Sequencer::new(mempool))); // Wrap in Mutex

    let handle = start_rpc_server(sequencer.clone(), 3030).await?;

    handle.stopped().await;
    Ok(())
}
