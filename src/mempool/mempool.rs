use alloy::consensus::TxEnvelope;
use alloy::primitives::keccak256;
use anyhow::{Context, Result};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::rpc_params;
use jsonrpsee::http_client::HttpClientBuilder;
use rocksdb::{Options, DB};
use serde_json;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Mempool {
    db: Arc<Mutex<DB>>,
    client_url: String,
}

impl Mempool {
    pub fn builder() -> MempoolBuilder {
        MempoolBuilder::default()
    }

    pub async fn add_transaction(&self, tx_hash: &str, tx_data: &TxEnvelope) -> Result<()> {
        let db = self.db.lock().await;
        db.put(
            tx_hash,
            serde_json::to_string(tx_data).context("Failed to serialize transaction data")?,
        )
        .context("Failed to add transaction to the database")?;
        Ok(())
    }

    pub async fn delete_transaction(&self, tx_hash: &str) -> Result<()> {
        let db = self.db.lock().await;
        db.delete(tx_hash).context(format!(
            "Failed to delete transaction with hash: {}",
            tx_hash
        ))?;
        println!(
            "Transaction with hash {} deleted from the database.",
            tx_hash
        );
        Ok(())
    }

    pub async fn store_batch(&self, batch: Vec<TxEnvelope>, port: u16) -> Result<()> {
        let db = self.db.lock().await;

        for transaction in &batch {
            let serialized_tx =
                serde_json::to_string(transaction).context("Failed to serialize transaction")?;
            let tx_hash = format!("{:x}", keccak256(serialized_tx.clone()));
            db.put(&tx_hash, serialized_tx)
                .context("Failed to store transaction in the database")?;
        }

        self.log_batch(&batch).await;

        self.send_batch_to_server(batch, port).await?;
        Ok(())
    }

    pub async fn send_batch_to_server(&self, batch: Vec<TxEnvelope>, port: u16) -> Result<()> {
        let client = HttpClientBuilder::default().build(&self.client_url)?;

        let result: Result<String, jsonrpsee::core::Error> = client
            .request("twrep_sendTransaction", rpc_params![batch])
            .await;

        match result {
            Ok(response) => {
                println!("Response from server on port {}: {:?}", port, response);
            }
            Err(err) => {
                eprintln!("Error sending batch: {:?}", err);
            }
        }

        Ok(())
    }

    async fn log_batch(&self, batch: &[TxEnvelope]) {
        match serde_json::to_string(batch) {
            Ok(serialized_batch) => {
                println!("Stored batch of transactions: {}", serialized_batch);
            }
            Err(e) => {
                eprintln!("Failed to serialize batch: {:?}", e);
            }
        }
    }
}

#[derive(Default)]
pub struct MempoolBuilder {
    path: Option<String>,
    client_url: Option<String>,
}

impl MempoolBuilder {
    pub fn path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }

    pub fn client_url(mut self, url: &str) -> Self {
        self.client_url = Some(url.to_string());
        self
    }

    pub fn build(self) -> Result<Mempool> {
        let path = self.path.context("Database path not specified")?;
        let client_url = self.client_url.context("Client URL not specified")?;

        let mut opts = Options::default();
        opts.create_if_missing(true);

        let db =
            DB::open(&opts, &path).context(format!("Failed to open database at path: {}", path))?;

        Ok(Mempool {
            db: Arc::new(Mutex::new(db)),
            client_url,
        })
    }
}
