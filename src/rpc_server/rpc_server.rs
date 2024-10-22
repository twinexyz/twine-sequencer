use crate::sequencer::sequencer::Sequencer;
use alloy::consensus::TxEnvelope;
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use anyhow::{Context, Result};
use jsonrpsee::server::{RpcModule, ServerBuilder, ServerHandle};
use jsonrpsee::types::{ErrorObject, ErrorObjectOwned};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

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

        // Register eth_sendTransaction method
        module
            .register_async_method("eth_sendTransaction", move |params, _| {
                let sequencer = Arc::clone(&sequencer_clone);
                async move {
                    let signed_tx: Result<TxEnvelope, ErrorObjectOwned> =
                        params.one().map_err(|e| {
                            ErrorObject::owned(
                                1,
                                format!("Failed to parse transaction: {:?}", e),
                                None::<()>,
                            )
                        });

                    match signed_tx {
                        Ok(tx) => {
                            info!("Received signed transaction: {:?}", tx);

                            let mut sequencer_lock = sequencer.lock().await; // Lock the mutex
                            match sequencer_lock.send_transaction(tx).await {
                                Ok(tx_hash) => Ok(tx_hash),
                                Err(e) => Err(ErrorObject::owned(
                                    2,
                                    format!("Failed to send transaction: {:?}", e),
                                    None::<()>,
                                )),
                            }
                        }
                        Err(e) => Err(e),
                    }
                }
            })
            .context("Failed to register eth_sendTransaction method")?;

        let sequencer_clone = Arc::clone(&self.sequencer);

        module.register_async_method("eth_getBalance", move |params, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                // Extract parameters directly as a flat array
                let params: Vec<serde_json::Value> = params.one().map_err(|e| {
                    ErrorObject::owned(
                        1,
                        format!("Failed to parse parameters: {:?}", e),
                        None::<()>,
                    )
                })?;
        
                // Log the received parameters
                info!("Received parameters for eth_getBalance: {:?}", params);
        
                // Ensure there are exactly two parameters
                if params.len() != 2 {
                    return Err(ErrorObject::owned(
                        5,
                        "Invalid number of parameters; expected 2".to_string(),
                        None::<()>,
                    ));
                }
        
                // Extract the address string and block parameter
                let address_str = params[0].as_str().ok_or_else(|| {
                    ErrorObject::owned(
                        3,
                        "First parameter must be a string (address)".to_string(),
                        None::<()>,
                    )
                })?;
        
                let block_param = params[1].as_str().ok_or_else(|| {
                    ErrorObject::owned(
                        6,
                        "Second parameter must be a string (block)".to_string(),
                        None::<()>,
                    )
                })?;
        
                // Parse the address from the string
                let address: Address = address_str.parse().map_err(|_| {
                    ErrorObject::owned(4, "Invalid address format".to_string(), None::<()>)
                })?;
        
                info!("Address parsed successfully: {}", address);
        
                // Acquire the sequencer lock and get the provider
                let sequencer_lock = sequencer.lock().await;
                let provider = sequencer_lock.get_provider();
        
                // Use the provider to get the balance
                let balance: U256 = provider.get_balance(address).await.map_err(|e| {
                    ErrorObject::owned(2, format!("Failed to get balance: {:?}", e), None::<()>)
                })?;
        
                info!("Balance retrieved: {}", balance);
        
                // Return the balance in hexadecimal format
                Ok::<String, ErrorObject>(format!("{:x}", balance))
            }
        })
        .context("Failed to register eth_getBalance method")?;
        

        let handle: ServerHandle = server
            .start(module)
            .context("Failed to start JSON-RPC server")?;

        info!("JSON-RPC server running on 127.0.0.1:{}", self.port);
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

        Ok(RpcServer { sequencer, port })
    }
}
