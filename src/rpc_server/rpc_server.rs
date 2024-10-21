use jsonrpsee::server::{RpcModule, ServerBuilder, ServerHandle};
use jsonrpsee::types::{ErrorObject, ErrorObjectOwned}; 
use std::sync::Arc;
use anyhow::{Context, Result};
use alloy::consensus::TxEnvelope;
use crate::sequencer::sequencer::Sequencer; 
use tokio::sync::Mutex;

pub struct RpcServer {
    sequencer: Arc<Mutex<Sequencer>>,
    port: u16,
}

impl RpcServer {
    pub fn builder() -> RpcServerBuilder {
        RpcServerBuilder::default()
    }

    pub async fn start(self) -> Result<ServerHandle> {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], self.port));
        let server = ServerBuilder::default()
            .build(addr)
            .await
            .context("Failed to build the JSON-RPC server")?;

        let mut module = RpcModule::new(());

        let sequencer_clone = Arc::clone(&self.sequencer);
        module
            .register_async_method("eth_sendTransaction", move |params, _| {
                let sequencer = Arc::clone(&sequencer_clone);
                async move {
                    let signed_tx: Result<TxEnvelope, ErrorObjectOwned> = params.one().map_err(|e| {
                        ErrorObject::owned(1, format!("Failed to parse transaction: {:?}", e),None::<()>) 
                    });

                    match signed_tx {
                        Ok(tx) => {
                            println!("Received signed transaction: {:?}", tx);

                            let mut sequencer = sequencer.lock().await;
                            match sequencer.send_transaction(tx).await {
                                Ok(tx_hash) => Ok(tx_hash),
                                Err(e) => Err(ErrorObject::owned(2, format!("Failed to send transaction: {:?}", e), None::<()>)), 
                            }
                        }
                        Err(e) => Err(e), 
                    }
                }
            })
            .context("Failed to register eth_sendTransaction method")?;

        let handle: ServerHandle = server.start(module).context("Failed to start JSON-RPC server")?;

        println!("JSON-RPC server running on 127.0.0.1:{}", self.port);
        Ok(handle)
    }
}

#[derive(Default)]
pub struct RpcServerBuilder {
    sequencer: Option<Arc<Mutex<Sequencer>>>,
    port: Option<u16>,
}

impl RpcServerBuilder {
    pub fn sequencer(mut self, sequencer: Arc<Mutex<Sequencer>>) -> Self {
        self.sequencer = Some(sequencer);
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn build(self) -> Result<RpcServer> {
        let sequencer = self.sequencer.context("Sequencer not provided")?;
        let port = self.port.context("Port not provided")?;

        Ok(RpcServer {
            sequencer,
            port,
        })
    }
}
