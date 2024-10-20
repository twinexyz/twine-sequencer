use rocksdb::{DB, Options};
use serde_json;
use std::sync::Arc;
use tokio::sync::Mutex;
use alloy::primitives::keccak256;
use alloy::rpc::types::TransactionRequest;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{HttpClientBuilder, HttpClient};
use jsonrpsee::core::rpc_params;

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

    pub async fn delete_transaction(&self, tx_hash: &str) {
        let db = self.db.lock().await;
        match db.delete(tx_hash) {
            Ok(_) => {
                println!("Transaction with hash {} deleted from the database.", tx_hash);
            }
            Err(e) => {
                eprintln!("Failed to delete transaction {}: {:?}", tx_hash, e);
            }
        }
    }
    
    pub async fn store_batch(&self, batch: Vec<TransactionRequest>, server_port: u16) {
        let db = self.db.lock().await;
    
        // Serialize and store each TransactionRequest in the batch
        for transaction in &batch {
            let serialized_tx = serde_json::to_string(transaction).unwrap(); // Serialize TransactionRequest directly
            let tx_hash = format!("{:x}", keccak256(serialized_tx.clone()));
            db.put(&tx_hash, serialized_tx).unwrap(); // Store the transaction
        }
    
        // Log the batch after storing
        self.log_batch(&batch).await;
    
        // Send the batch to the server
        if let Err(e) = self.send_batch_to_server(batch, server_port).await {
            eprintln!("Failed to send batch to server: {}", e);
        }
    }
    

    pub async fn send_batch_to_server(&self, batch: Vec<TransactionRequest>, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        // Create the HTTP client to connect to the JSON-RPC server
        let client = HttpClientBuilder::default().build(format!("http://127.0.0.1:{}", port))?;
    
        // Specify the expected response type here
        let result: Result<String, jsonrpsee::core::Error> = client.request("twrep_sendTransaction", rpc_params![batch]).await;
    
        // Handle the response from the server
        match result {
            Ok(response) => {
                println!("Response from server: {:?}", response);
            }
            Err(err) => {
                eprintln!("Error sending batch: {:?}", err);
            }
        }
    
        Ok(())
    }
    
    
    async fn log_batch(&self, batch: &[TransactionRequest]) {
        println!("Stored batch of transactions:");
        
        // Log the batch directly
        match serde_json::to_string(batch) {
            Ok(serialized_batch) => {
                println!("Batch Data: {}", serialized_batch);
            }
            Err(e) => {
                eprintln!("Failed to serialize batch: {:?}", e);
            }
        }
    }
    
}
