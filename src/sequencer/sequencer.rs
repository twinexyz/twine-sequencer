use jsonrpsee::core::RpcResult;
use jsonrpsee::types::error::ErrorObjectOwned;
use alloy::rpc::types::TransactionRequest;
use alloy::primitives::keccak256;

pub struct Sequencer {
    mempool: crate::mempool::mempool::Mempool,
}

impl Sequencer {
    pub fn new(mempool: crate::mempool::mempool::Mempool) -> Self {
        Sequencer { mempool }
    }

    pub async fn send_transaction(&self, tx: TransactionRequest) -> RpcResult<String> {
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

        let tx_hash = format!("{:x}", keccak256(serde_json::to_string(&tx).unwrap()));
        self.mempool.add_transaction(&tx_hash, &tx).await;
        self.mempool.print_all_transactions().await;

        Ok(tx_hash)
    }
}
