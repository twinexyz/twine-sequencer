use crate::mempool::mempool::Mempool; 
use alloy::rpc::types::TransactionRequest;
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::error::ErrorObjectOwned;
use std::collections::VecDeque;
use alloy::primitives::keccak256;

pub struct Sequencer {
    mempool: Mempool,
    pending_transactions: VecDeque<TransactionRequest>,
}

impl Sequencer {    
    pub fn new(mempool: Mempool) -> Self {
        Self {
            mempool,
            pending_transactions: VecDeque::new(),
        }
    }
    

    pub async fn send_transaction(&mut self, tx: TransactionRequest) -> RpcResult<String> {
        // Validation (same as before)
        if tx.to.is_none() {
            return Err(ErrorObjectOwned::owned(
                400, "Missing 'to' field", None::<()>
            ).into());
        }
    
        if let (Some(max_fee_per_gas), Some(max_priority_fee_per_gas)) = 
            (tx.max_fee_per_gas, tx.max_priority_fee_per_gas) {
            if max_fee_per_gas < max_priority_fee_per_gas {
                return Err(ErrorObjectOwned::owned(
                    400, "'max_fee_per_gas' must be >= 'max_priority_fee_per_gas'", None::<()>
                ).into());
            }
        } else {
            return Err(ErrorObjectOwned::owned(
                400, "Missing EIP-1559 fee fields", None::<()>
            ).into());
        }
    
        // Append the transaction directly to the pending_transactions
        self.pending_transactions.push_back(tx.clone());
    
        // Process a batch if we have enough transactions
        if self.pending_transactions.len() >= 3 {
            let batch: Vec<TransactionRequest> = self.pending_transactions.drain(..3).collect();
        
            // Store the batch in RocksDB
            self.mempool.store_batch(batch.clone(),45001).await;
    
            // Delete each transaction from the database by its hash if needed
            for transaction in &batch {
                let tx_hash = format!("{:x}", keccak256(serde_json::to_string(transaction).unwrap()));
                self.mempool.delete_transaction(&tx_hash).await;
            }
        }
    
        let tx_hash = format!("{:x}", keccak256(serde_json::to_string(&tx).unwrap()));
        self.mempool.add_transaction(&tx_hash, &tx).await;
    
        Ok(tx_hash)
    }
    
}
