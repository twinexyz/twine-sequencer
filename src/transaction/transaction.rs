use alloy::rpc::types::TransactionRequest;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionData {
    pub tx: TransactionRequest,
}

impl TransactionData {
    pub fn new(tx: &TransactionRequest) -> Self {  
        TransactionData { 
            tx: tx.clone(), 
        }
    }
}
