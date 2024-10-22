use crate::mempool::mempool::Mempool;
use alloy::consensus::TxEnvelope;
use jsonrpsee::types::ErrorObject; 
use std::collections::VecDeque;
use alloy::primitives::keccak256;
use anyhow::{Context, Result};
use tokio::sync::Mutex;
use std::sync::Arc;
use alloy::providers::RootProvider; 
use alloy::pubsub::PubSubFrontend;

pub struct Sequencer {
    mempool: Arc<Mutex<Mempool>>,
    pending_transactions: VecDeque<TxEnvelope>,
    batch_size: usize,
    server_port: u16,
    provider: RootProvider<PubSubFrontend>, // Use specific provider
}

impl Sequencer {
    pub async fn send_transaction(&mut self, tx: TxEnvelope) -> Result<String> {
        self.pending_transactions.push_back(tx.clone());
    
        if self.pending_transactions.len() >= self.batch_size {
            let batch: Vec<TxEnvelope> = self.pending_transactions.drain(..self.batch_size).collect();
    
            self.mempool
                .lock()
                .await
                .store_batch(batch.clone(), self.server_port)
                .await
                .context("Failed to store batch in RocksDB") 
                .map_err(|e| ErrorObject::owned(1, e.to_string(), None::<()>) )?; 
    
            for transaction in &batch {
                let tx_hash = format!("{:x}", keccak256(serde_json::to_string(transaction).unwrap()));
                self.mempool.lock().await.delete_transaction(&tx_hash).await.map_err(|e| {
                    ErrorObject::owned(1, e.to_string(), None::<()>) 
                })?;
            }
        }
    
        let tx_hash = format!("{:x}", keccak256(serde_json::to_string(&tx).unwrap()));
        self.mempool.lock().await.add_transaction(&tx_hash, &tx).await.map_err(|e| {
            ErrorObject::owned(1, e.to_string(), None::<()>) 
        })?;
    
        Ok(tx_hash)
    }
    
    pub fn get_provider(&self) -> &RootProvider<PubSubFrontend> {
        &self.provider
    }

    pub fn builder() -> SequencerBuilder {
        SequencerBuilder::default()
    }
}

#[derive(Default)]
pub struct SequencerBuilder { 
    mempool: Option<Arc<Mutex<Mempool>>>,
    batch_size: Option<usize>,
    server_port: Option<u16>,
    provider: Option<RootProvider<PubSubFrontend>>, // Use specific provider
}

impl SequencerBuilder { 
    pub fn mempool(mut self, mempool: Arc<Mutex<Mempool>>) -> Self {
        self.mempool = Some(mempool);
        self
    }

    pub fn batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = Some(batch_size);
        self
    }

    pub fn server_port(mut self, port: u16) -> Self {
        self.server_port = Some(port);
        self
    }

    pub fn provider(mut self, provider: RootProvider<PubSubFrontend>) -> Self {
        self.provider = Some(provider);
        self
    }

    pub fn build(self) -> Result<Sequencer> {
        let mempool = self.mempool.context("Mempool not provided")?;
        let batch_size = self.batch_size.context("Batch size not provided")?;
        let server_port = self.server_port.context("Server port not provided")?;
        let provider = self.provider.context("Provider not provided")?;

        Ok(Sequencer {
            mempool,
            pending_transactions: VecDeque::new(),
            batch_size,
            server_port,
            provider,
        })
    }
}
