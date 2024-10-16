use alloy::rpc::types::TransactionRequest;
use rocksdb::{DB, Options};
use serde_json;
use std::sync::Arc;
use tokio::sync::Mutex;

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

    pub async fn add_transaction(&self, tx_hash: &str, tx_data: &TransactionRequest) {
        let db = self.db.lock().await;
        db.put(tx_hash, serde_json::to_string(tx_data).unwrap()).unwrap();
    }

    pub async fn print_all_transactions(&self) {
        let db = self.db.lock().await;
        let iter = db.iterator(rocksdb::IteratorMode::Start);
    
        for result in iter {
            match result {
                Ok((key, value)) => {
                    let tx_hash = String::from_utf8_lossy(&key).to_string();
                    let tx_data = String::from_utf8_lossy(&value).to_string();
                    println!("Transaction Hash: {}\nTransaction Data: {}\n", tx_hash, tx_data);
                }
                Err(e) => {
                    eprintln!("Error while iterating: {:?}", e);
                }
            }
        }
    }
}
