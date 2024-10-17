use rocksdb::{DB, Options};
use serde_json;
use std::sync::Arc;
use tokio::sync::Mutex;
use alloy::primitives::keccak256;
use crate::transaction::transaction::TimestampedTransaction;

pub struct Mempool {
    db: Arc<Mutex<DB>>,
}

impl Mempool {
    pub fn new(path: &str) -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path).unwrap();
        Self {
            db: Arc::new(Mutex::new(db)),
        }
    }

    pub async fn add_transaction(&self, tx_hash: &str, tx_data: &TimestampedTransaction) {
        let db = self.db.lock().await;
        db.put(tx_hash, serde_json::to_string(tx_data).unwrap()).unwrap();
    }
    
    pub async fn store_batch(&self, batch: Vec<TimestampedTransaction>) {
        let db = self.db.lock().await;
    
        for transaction in &batch {
            let serialized_tx = serde_json::to_string(&transaction).unwrap();
            let tx_hash = format!("{:x}", keccak256(serialized_tx.clone()));
            db.put(&tx_hash, serialized_tx).unwrap();
        }

        // Log the batch after storing
        self.log_batch(&batch).await;
    }

    async fn log_batch(&self, batch: &[TimestampedTransaction]) {
        println!("Stored batch of transactions:");
        for transaction in batch {
            let serialized_tx = serde_json::to_string(transaction).unwrap();
            let tx_hash = format!("{:x}", keccak256(serialized_tx.clone()));
            println!("Transaction Hash in batch: {}", tx_hash);
            println!("Transaction Data: {:?}", transaction);
        }
    }

}
