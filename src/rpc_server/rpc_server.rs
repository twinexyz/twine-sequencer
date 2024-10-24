use crate::rpc_server::eth_methods::register_eth_methods;
use crate::{
    rpc_server::sequencer_methods::register_sequencer_methods, sequencer::sequencer::Sequencer,
};
use anyhow::{Context, Result};
use jsonrpsee::server::{RpcModule, ServerBuilder, ServerHandle};
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

        register_eth_methods(&mut module, Arc::clone(&self.sequencer))
            .context("Failed to register Ethereum methods")?;

        register_sequencer_methods(&mut module, Arc::clone(&self.sequencer))
            .context("Failed to register Sequencer methods")?;

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
