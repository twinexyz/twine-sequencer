use crate::mempool::mempool::Mempool;
use crate::rpc_server::rpc_server::RpcServer;
use crate::sequencer::sequencer::Sequencer;
use dotenv::dotenv;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing_subscriber;

mod mempool;
mod rpc_server;
mod sequencer;

use alloy::providers::{ProviderBuilder, RootProvider, WsConnect};
use alloy::pubsub::PubSubFrontend;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    dotenv().ok();
    let api_key = env::var("INFURA_PROJECT_ID").expect("INFURA_PROJECT_ID not in .env");

    let eth_rpc_url = format!("wss://mainnet.infura.io/ws/v3/{}", api_key);

    let eth_ws = WsConnect::new(&eth_rpc_url);

    let ethereum_provider: RootProvider<PubSubFrontend> =
        ProviderBuilder::new().on_ws(eth_ws).await?;

    let mempool = Mempool::builder()
        .path("./mempool_db")
        .client_url("http://localhost:45001")
        .build()?;

    let mempool = Arc::new(Mutex::new(mempool));

    let sequencer = Sequencer::builder()
        .mempool(Arc::clone(&mempool))
        .batch_size(3)
        .server_port(3030)
        .provider(ethereum_provider)
        .build()?;

    let sequencer = Arc::new(Mutex::new(sequencer));

    let rpc_server = RpcServer::builder()
        .sequencer(sequencer)
        .port(3030)
        .build()?;

    let _handle = rpc_server.start().await?;

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C signal handler");

    println!("Shutting down gracefully...");

    Ok(())
}