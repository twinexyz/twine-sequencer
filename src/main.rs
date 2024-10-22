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

use alloy::providers::{ProviderBuilder, WsConnect, RootProvider};
use alloy::pubsub::PubSubFrontend;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Load environment variables from .env file
    dotenv().ok();
    let api_key = env::var("INFURA_PROJECT_ID").expect("INFURA_PROJECT_ID not in .env");

    // Construct the Ethereum WebSocket URL
    let eth_rpc_url = format!("wss://mainnet.infura.io/ws/v3/{}", api_key);

    // Create a WebSocket connection for Ethereum provider
    let eth_ws = WsConnect::new(&eth_rpc_url);
    
    // Create the Ethereum provider
    let ethereum_provider: RootProvider<PubSubFrontend> = 
        ProviderBuilder::new().on_ws(eth_ws).await?;

    // Initialize the mempool
    let mempool = Mempool::builder()
        .path("./mempool_db")
        .client_url("http://localhost:45001")
        .build()?;

    let mempool = Arc::new(Mutex::new(mempool));

    // Initialize the sequencer with the provider
    let sequencer = Sequencer::builder()
        .mempool(Arc::clone(&mempool))
        .batch_size(3)
        .server_port(3030)
        .provider(ethereum_provider) // Set the provider here
        .build()?;

    let sequencer = Arc::new(Mutex::new(sequencer));

    // Start the JSON-RPC server
    let rpc_server = RpcServer::builder()
        .sequencer(sequencer)
        .port(3030)
        .build()?;

    let _handle = rpc_server.start().await?; // Start the RPC server

    // Keep the server running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}



