use alloy::rpc::types::TransactionRequest;
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize,Debug, Clone)]
pub struct TimestampedTransaction {
    pub tx: TransactionRequest,
    pub timestamp: u64,
}

impl TimestampedTransaction {
    pub fn new(tx: &TransactionRequest) -> Self {  
        let start = SystemTime::now();
        let timestamp = start.duration_since(UNIX_EPOCH).unwrap().as_secs();
        TimestampedTransaction { 
            tx: tx.clone(), 
            timestamp 
        }
    }
}
