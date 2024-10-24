use crate::sequencer::sequencer::Sequencer;
use alloy::consensus::TxEnvelope;
use alloy::hex;
use alloy::primitives::{Address, B256, U256};
use alloy::providers::Provider;
use alloy::rpc::types::{Filter, Log, TransactionRequest};
use anyhow::{Context, Result};
use jsonrpsee::types::{ErrorObject, ErrorObjectOwned};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

pub fn register_eth_methods(
    module: &mut jsonrpsee::RpcModule<()>,
    sequencer: Arc<Mutex<Sequencer>>,
) -> Result<()> {
    let sequencer_clone = Arc::clone(&sequencer);

    // Register eth_sendTransaction method
    module
        .register_async_method("eth_sendTransaction", move |params, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                let signed_tx: Result<TxEnvelope, ErrorObjectOwned> = params.one().map_err(|e| {
                    ErrorObject::owned(
                        1,
                        format!("Failed to parse transaction: {:?}", e),
                        None::<()>,
                    )
                });

                match signed_tx {
                    Ok(tx) => {
                        info!("Received signed transaction: {:?}", tx);

                        let mut sequencer_lock = sequencer.lock().await; // Lock the mutex
                        match sequencer_lock.send_transaction(tx).await {
                            Ok(tx_hash) => Ok(tx_hash),
                            Err(e) => Err(ErrorObject::owned(
                                2,
                                format!("Failed to send transaction: {:?}", e),
                                None::<()>,
                            )),
                        }
                    }
                    Err(e) => Err(e),
                }
            }
        })
        .context("Failed to register eth_sendTransaction method")?;

    let sequencer_clone = Arc::clone(&sequencer);

    module
        .register_async_method("eth_sendRawTransaction", move |params, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                let params: Vec<serde_json::Value> = params.one().map_err(|e| {
                    ErrorObject::owned(
                        1,
                        format!("Failed to parse parameters: {:?}", e),
                        None::<()>,
                    )
                })?;

                info!(
                    "Received parameters for eth_sendRawTransaction: {:?}",
                    params
                );

                if params.len() != 1 {
                    return Err(ErrorObject::owned(
                        5,
                        "Invalid number of parameters; expected 1".to_string(),
                        None::<()>,
                    ));
                }

                let raw_tx_hex = params[0].as_str().ok_or_else(|| {
                    ErrorObject::owned(
                        3,
                        "First parameter must be a string (raw transaction hex)".to_string(),
                        None::<()>,
                    )
                })?;

                let encoded_tx: &[u8] = &hex::decode(raw_tx_hex).map_err(|_| {
                    ErrorObject::owned(
                        4,
                        "Invalid hex format for raw transaction".to_string(),
                        None::<()>,
                    )
                })?;

                info!("Raw transaction decoded successfully");

                let mut sequencer_lock = sequencer.lock().await;
                match sequencer_lock.send_raw_transaction(encoded_tx).await {
                    Ok(tx_hash) => {
                        info!("Transaction sent successfully, tx_hash: {:?}", tx_hash);
                        Ok(tx_hash.to_string())
                    }
                    Err(e) => Err(ErrorObject::owned(
                        2,
                        format!("Failed to send raw transaction: {:?}", e),
                        None::<()>,
                    )),
                }
            }
        })
        .context("Failed to register eth_sendRawTransaction method")?;

    let sequencer_clone = Arc::clone(&sequencer);
    // Register eth_getBalance method
    module
        .register_async_method("eth_getBalance", move |params, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                let params: Vec<serde_json::Value> = params.parse().map_err(|e| {
                    ErrorObject::owned(
                        1,
                        format!("Failed to parse parameters: {:?}", e),
                        None::<()>,
                    )
                })?;

                if params.len() != 2 {
                    return Err(ErrorObject::owned(
                        5,
                        "Invalid number of parameters; expected 2".to_string(),
                        None::<()>,
                    ));
                }

                let address_str = params[0].as_str().ok_or_else(|| {
                    ErrorObject::owned(
                        3,
                        "First parameter must be a string (address)".to_string(),
                        None::<()>,
                    )
                })?;

                let address: Address = address_str.parse().map_err(|_| {
                    ErrorObject::owned(4, "Invalid address format".to_string(), None::<()>)
                })?;

                info!("Address parsed successfully: {}", address);

                let block_tag = params[1].as_str().ok_or_else(|| {
                    ErrorObject::owned(
                        3,
                        "Second parameter must be a string (block)".to_string(),
                        None::<()>,
                    )
                })?;

                info!("Block parameter parsed: {}", block_tag);

                let sequencer_lock = sequencer.lock().await;
                let provider = sequencer_lock.get_provider();

                let balance: U256 = provider.get_balance(address).await.map_err(|e| {
                    ErrorObject::owned(2, format!("Failed to get balance: {:?}", e), None::<()>)
                })?;

                info!("Balance retrieved: {}", balance);

                Ok::<String, ErrorObject>(format!("{:x}", balance))
            }
        })
        .context("Failed to register eth_getBalance method")?;

    //Register eth_accounts
    let sequencer_clone = Arc::clone(&sequencer);

    module
        .register_async_method("eth_accounts", move |_, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                let sequencer_lock = sequencer.lock().await;
                let provider = sequencer_lock.get_provider();

                let accounts: Vec<Address> = provider.get_accounts().await.map_err(|e| {
                    ErrorObject::owned(2, format!("Failed to get accounts: {:?}", e), None::<()>)
                })?;

                let accounts_hex: Vec<String> = accounts
                    .iter()
                    .map(|account| format!("{:x}", account))
                    .collect();

                Ok::<Vec<String>, ErrorObject>(accounts_hex)
            }
        })
        .context("Failed to register eth_accounts method")?;

    //Register eth_blockNumber
    let sequencer_clone = Arc::clone(&sequencer);
    module
        .register_async_method("eth_blockNumber", move |_, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                let sequencer_lock = sequencer.lock().await;
                let provider = sequencer_lock.get_provider();

                let block_number: u64 = provider.get_block_number().await.map_err(|e| {
                    ErrorObject::owned(
                        2,
                        format!("Failed to get block number: {:?}", e),
                        None::<()>,
                    )
                })?;

                Ok::<u64, ErrorObject>(block_number)
            }
        })
        .context("Failed to register eth_blockNumber method")?;

    //Register eth_chainId
    let sequencer_clone = Arc::clone(&sequencer);
    module
        .register_async_method("eth_chainId", move |_, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                let sequencer_lock = sequencer.lock().await;
                let provider = sequencer_lock.get_provider();

                let chain_id: u64 = provider.get_chain_id().await.map_err(|e| {
                    ErrorObject::owned(2, format!("Failed to get chain ID: {:?}", e), None::<()>)
                })?;

                Ok::<u64, ErrorObject>(chain_id)
            }
        })
        .context("Failed to register eth_chainId method")?;

    //Register eth_gasPrice
    let sequencer_clone = Arc::clone(&sequencer);
    module
        .register_async_method("eth_gasPrice", move |_params, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                let sequencer_lock = sequencer.lock().await;
                let provider = sequencer_lock.get_provider();

                let gas_price = provider.get_gas_price().await.map_err(|e| {
                    ErrorObject::owned(2, format!("Failed to get gas price: {:?}", e), None::<()>)
                })?;

                let gas_price_hex = format!("0x{:x}", gas_price);

                Ok::<String, ErrorObject>(gas_price_hex)
            }
        })
        .context("Failed to register eth_gasPrice method")?;

    //Register eth_getCode
    let sequencer_clone = Arc::clone(&sequencer);
    module
        .register_async_method("eth_getCode", move |params, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                let params: Vec<serde_json::Value> = params.parse().map_err(|e| {
                    ErrorObject::owned(
                        1,
                        format!("Failed to parse parameters: {:?}", e),
                        None::<()>,
                    )
                })?;

                let address_str = params[0].as_str().ok_or_else(|| {
                    ErrorObject::owned(
                        4,
                        "First parameter must be a string (address)".to_string(),
                        None::<()>,
                    )
                })?;

                let address = Address::from_str(address_str).unwrap();

                let sequencer_lock = sequencer.lock().await;
                let provider = sequencer_lock.get_provider();

                let code = provider.get_code_at(address).await.map_err(|e| {
                    ErrorObject::owned(2, format!("Failed to get code: {:?}", e), None::<()>)
                })?;

                Ok::<String, ErrorObject>(code.to_string())
            }
        })
        .context("Failed to register eth_getCode method")?;

    let sequencer_clone = Arc::clone(&sequencer);
    module
        .register_async_method("eth_getTransactionByHash", move |params, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                let params: Vec<serde_json::Value> = params.parse().map_err(|e| {
                    ErrorObject::owned(
                        1,
                        format!("Failed to parse parameters: {:?}", e),
                        None::<()>,
                    )
                })?;

                if params.len() != 1 {
                    return Err(ErrorObject::owned(
                        5,
                        "Invalid number of parameters; expected 1".to_string(),
                        None::<()>,
                    ));
                }

                let tx_hash_str = params[0].as_str().ok_or_else(|| {
                    ErrorObject::owned(
                        4,
                        "First parameter must be a string (transaction hash)".to_string(),
                        None::<()>,
                    )
                })?;

                let tx_hash = B256::from_str(tx_hash_str).map_err(|_| {
                    ErrorObject::owned(6, "Invalid transaction hash format".to_string(), None::<()>)
                })?;

                let sequencer_lock = sequencer.lock().await;
                let provider = sequencer_lock.get_provider();

                let transaction = provider
                    .get_transaction_by_hash(tx_hash)
                    .await
                    .map_err(|e| {
                        ErrorObject::owned(
                            2,
                            format!("Failed to get transaction: {:?}", e),
                            None::<()>,
                        )
                    })?;

                Ok(transaction)
            }
        })
        .context("Failed to register eth_getTransactionByHash method")?;

    let sequencer_clone = Arc::clone(&sequencer);
    module
        .register_async_method("eth_getTransactionReceipt", move |params, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                let params: Vec<serde_json::Value> = params.parse().map_err(|e| {
                    ErrorObject::owned(
                        1,
                        format!("Failed to parse parameters: {:?}", e),
                        None::<()>,
                    )
                })?;

                if params.len() != 1 {
                    return Err(ErrorObject::owned(
                        5,
                        "Invalid number of parameters; expected 1".to_string(),
                        None::<()>,
                    ));
                }

                let tx_hash_str = params[0].as_str().ok_or_else(|| {
                    ErrorObject::owned(
                        4,
                        "First parameter must be a string (transaction hash)".to_string(),
                        None::<()>,
                    )
                })?;

                let tx_hash = B256::from_str(tx_hash_str).map_err(|_| {
                    ErrorObject::owned(6, "Invalid transaction hash format".to_string(), None::<()>)
                })?;

                let sequencer_lock = sequencer.lock().await;
                let provider = sequencer_lock.get_provider();

                let receipt = provider
                    .get_transaction_receipt(tx_hash)
                    .await
                    .map_err(|e| {
                        ErrorObject::owned(
                            2,
                            format!("Failed to get transaction receipt: {:?}", e),
                            None::<()>,
                        )
                    })?;

                Ok(receipt)
            }
        })
        .context("Failed to register eth_getTransactionReceipt method")?;

    let sequencer_clone = Arc::clone(&sequencer);
    module
        .register_async_method("eth_maxPriorityFeePerGas", move |_, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                let sequencer_lock = sequencer.lock().await;
                let provider = sequencer_lock.get_provider();

                let max_priority_fee =
                    provider.get_max_priority_fee_per_gas().await.map_err(|e| {
                        ErrorObject::owned(
                            2,
                            format!("Failed to get max priority fee per gas: {:?}", e),
                            None::<()>,
                        )
                    })?;

                Ok::<u128, ErrorObject>(max_priority_fee)
            }
        })
        .context("Failed to register eth_maxPriorityFeePerGas method")?;

    let sequencer_clone = Arc::clone(&sequencer);
    module
        .register_async_method("eth_call", move |params, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                let params: Vec<serde_json::Value> = params.parse().map_err(|e| {
                    ErrorObject::owned(
                        1,
                        format!("Failed to parse parameters: {:?}", e),
                        None::<()>,
                    )
                })?;

                if params.len() != 2 {
                    return Err(ErrorObject::owned(
                        5,
                        "Invalid number of parameters; expected 2".to_string(),
                        None::<()>,
                    ));
                }

                let tx_value = &params[0];

                let signed_tx: TransactionRequest = serde_json::from_value(tx_value.clone())
                    .map_err(|e| {
                        ErrorObject::owned(
                            4,
                            format!("Failed to parse transaction: {:?}", e),
                            None::<()>,
                        )
                    })?;

                let sequencer_lock = sequencer.lock().await;
                let provider = sequencer_lock.get_provider();

                let result = provider.call(&signed_tx).await.map_err(|e| {
                    ErrorObject::owned(
                        2,
                        format!("Failed to execute eth_call: {:?}", e),
                        None::<()>,
                    )
                })?;

                Ok::<String, ErrorObject>(result.to_string())
            }
        })
        .context("Failed to register eth_call method")?;

    let sequencer_clone = Arc::clone(&sequencer);
    module
        .register_async_method("eth_getLogs", move |params, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                let filter: Filter = params.one().map_err(|e| {
                    ErrorObject::owned(
                        1,
                        format!("Failed to parse filter object: {:?}", e),
                        None::<()>,
                    )
                })?;

                let sequencer_lock = sequencer.lock().await;
                let provider = sequencer_lock.get_provider();

                let logs: Vec<Log> = provider.get_logs(&filter).await.map_err(|e| {
                    ErrorObject::owned(2, format!("Failed to fetch logs: {:?}", e), None::<()>)
                })?;

                Ok::<Vec<Log>, ErrorObject>(logs)
            }
        })
        .context("Failed to register eth_getLogs method")?;

    let sequencer_clone = Arc::clone(&sequencer);
    module
        .register_async_method("eth_getStorageAt", move |params, _| {
            let sequencer = Arc::clone(&sequencer_clone);
            async move {
                let (address, storage_position): (Address, U256) = params.parse().map_err(|e| {
                    ErrorObject::owned(
                        1,
                        format!("Failed to parse parameters: {:?}", e),
                        None::<()>,
                    )
                })?;

                let sequencer_lock = sequencer.lock().await;
                let provider = sequencer_lock.get_provider();

                let result = provider
                    .get_storage_at(address, storage_position)
                    .await
                    .map_err(|e| {
                        ErrorObject::owned(
                            2,
                            format!("Failed to fetch storage: {:?}", e),
                            None::<()>,
                        )
                    })?;

                Ok::<U256, ErrorObject>(result)
            }
        })
        .context("Failed to register eth_getStorageAt method")?;

    Ok(())
}
