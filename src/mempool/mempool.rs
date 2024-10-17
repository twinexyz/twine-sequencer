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
            let tx_hash = format!("{:x}", keccak256(serde_json::to_string(transaction).unwrap()));
            println!("transaction hashes in batch: {}", tx_hash);
            // Add more details to log as necessary
        }
    }
    
    // pub async fn print_all_transactions(&self) {
    //     let db = self.db.lock().await;
    //     let iter = db.iterator(rocksdb::IteratorMode::Start);
    
    //     for result in iter {
    //         match result {
    //             Ok((key, value)) => {
    //                 let tx_hash = String::from_utf8_lossy(&key).to_string();
    //                 let value_str = String::from_utf8_lossy(&value).to_string();
    
    //                 let tx_data: TimestampedTransaction = serde_json::from_str(&value_str)
    //                     .expect("Failed to deserialize transaction data");
    //                 // println!("Transaction Hash: {}\nTransaction Data: {:?}\n", tx_hash, tx_data);
    //                 println!("Transaction Hash: {}", tx_hash);

    //             }
    //             Err(e) => {
    //                 eprintln!("Error while iterating: {:?}", e);
    //             }
    //         }
    //     }
    // }


}
