use crate::mempool::mempool::Mempool; // Ensure the correct import
use crate::transaction::transaction::TimestampedTransaction;
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::error::ErrorObjectOwned;
use alloy::rpc::types::TransactionRequest;
use alloy::primitives::keccak256;
use std::collections::VecDeque;
use chrono::Utc;

pub struct Sequencer {
    mempool: crate::mempool::mempool::Mempool,
    pending_transactions: VecDeque<TimestampedTransaction>,
}

impl Sequencer {    
    pub fn new(mempool: Mempool) -> Self {
        Self {
            mempool,
            pending_transactions: VecDeque::new(),
        }
    }

    pub async fn send_transaction(&mut self, tx: TransactionRequest) -> RpcResult<String> {
        // Basic validation for 'to' field
        if tx.to.is_none() {
            return Err(ErrorObjectOwned::owned(
                400, "Missing 'to' field", None::<()>
            ).into());
        }
    
        // Validate max_fee_per_gas and max_priority_fee_per_gas for EIP-1559
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
    
        let timestamp = Utc::now().timestamp() as u64; // Get the current timestamp in seconds
        let timestamped_tx = TimestampedTransaction { tx: tx.clone(), timestamp };
    
        // Add the timestamped transaction to the queue
        self.pending_transactions.push_back(timestamped_tx.clone());
    
        // Check if we have enough transactions for a batch
        if self.pending_transactions.len() >= 3 {
            let batch: Vec<TimestampedTransaction> = self.pending_transactions.drain(..3).collect();
            self.mempool.store_batch(batch).await; // Store the batch in the mempool
            println!("here***************************");
        }
    
        let tx_hash = format!("{:x}", keccak256(serde_json::to_string(&timestamped_tx).unwrap()));
        self.mempool.add_transaction(&tx_hash, &timestamped_tx).await; // Store the individual transaction
        // self.mempool.print_all_transactions().await;
    
        Ok(tx_hash)
    }
    
    
    
}
