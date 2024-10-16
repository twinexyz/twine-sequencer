use jsonrpsee::core::RpcResult;
use jsonrpsee::server::{RpcModule, ServerBuilder, ServerHandle};
use jsonrpsee::types::error::ErrorObjectOwned;
use alloy::rpc::types::TransactionRequest;
use rocksdb::{DB, Options};
use std::sync::Arc;
use tokio::sync::Mutex;
use eyre::Result;
use std::net::SocketAddr;
use serde_json;
use alloy::primitives::keccak256;
struct Mempool {
    db: Arc<Mutex<DB>>,
}

impl Mempool {
    fn new(path: &str) -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path).unwrap();
        Self {
            db: Arc::new(Mutex::new(db)),
        }
    }


   

    async fn add_transaction(&self, tx_hash: &str, tx_data: &TransactionRequest) {
        let db = self.db.lock().await;
        db.put(tx_hash, serde_json::to_string(tx_data).unwrap()).unwrap();
    }

    async fn print_all_transactions(&self) {
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

struct Sequencer {
    mempool: Mempool,
}

impl Sequencer {
    fn new(mempool: Mempool) -> Self {
        Sequencer { mempool }
    }

    async fn send_transaction(&self, tx: TransactionRequest) -> RpcResult<String> {
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

        // Return the transaction hash
        Ok(tx_hash)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mempool = Mempool::new("mempool_db");
    let sequencer = Sequencer::new(mempool);
    let addr = SocketAddr::from(([127, 0, 0, 1], 8545));
    let server = ServerBuilder::default().build(addr).await?;

    let mut module = RpcModule::new(());
    let sequencer_arc = Arc::new(sequencer);

    let sequencer_clone = Arc::clone(&sequencer_arc);
    module
        .register_async_method("eth_sendTransaction", move |params, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                // Extract the TransactionRequest from the parameters
                let tx: TransactionRequest = params.one()?;
                sequencer.send_transaction(tx).await
            }
        })
        .unwrap();

    let handle: ServerHandle = server.start(module)?;

    println!("JSON-RPC server running on 127.0.0.1:8545");

    // Keep the server running
    handle.stopped().await;
    Ok(())
}
