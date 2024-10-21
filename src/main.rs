use crate::mempool::mempool::Mempool;
use crate::rpc_server::rpc_server::RpcServer;
use crate::sequencer::sequencer::Sequencer;
use std::sync::Arc;
use tokio::sync::Mutex;

mod mempool;
mod rpc_server;
mod sequencer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mempool = Mempool::builder()
        .path("./mempool_db")
        .client_url("http://localhost:45001")
        .build()?;

    let mempool = Arc::new(Mutex::new(mempool));

    let sequencer = Sequencer::builder()
        .mempool(Arc::clone(&mempool))
        .batch_size(3)
        .server_port(3030)
        .build()?;

    let sequencer = Arc::new(Mutex::new(sequencer));

    let rpc_server = RpcServer::builder()
        .sequencer(sequencer)
        .port(3030)
        .build()?;

    let _handle = rpc_server.start().await?;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}