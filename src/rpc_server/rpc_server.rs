use jsonrpsee::server::{RpcModule, ServerBuilder, ServerHandle};
use alloy::rpc::types::{TransactionRequest, Signature}; // Import necessary types
use std::sync::Arc; 
use anyhow::Result; 

use crate::sequencer::sequencer::Sequencer;
use tokio::sync::Mutex; 

pub async fn start_rpc_server(sequencer: Arc<Mutex<Sequencer>>, port: u16) -> Result<ServerHandle> {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let server = ServerBuilder::default().build(addr).await?;
    let mut module = RpcModule::new(());

    let sequencer_clone = Arc::clone(&sequencer);
    module
        .register_async_method("eth_sendTransaction", move |params, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                // Extract the signed transaction from the parameters
                let signed_tx: TransactionRequest = params.one()?; // Make sure to use the correct type

                // Log the entire signed transaction
                println!("Received signed transaction: {:?}", signed_tx);

                
                // Send the transaction using the sequencer
                let mut sequencer = sequencer.lock().await; 
                sequencer.send_transaction(signed_tx).await // Adjust if necessary to match your sequencer's method
            }
        })
        .unwrap();

    let handle: ServerHandle = server.start(module)?;

    println!("JSON-RPC server running on 127.0.0.1:{}", port);
    Ok(handle)
}
