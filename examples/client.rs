use alloy::{
    consensus::TxEnvelope,
    network::{EthereumWallet, TransactionBuilder},
    primitives::{address, Address, U256},
    providers::{Provider, ProviderBuilder, WalletProvider},
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
};
use eyre::Result;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClientBuilder;
use rand::Rng;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let signer: PrivateKeySigner =
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
            .trim_start_matches("0x")
            .parse()
            .expect("Error parsing private key");

    let wallet = EthereumWallet::from(signer.clone());
    let rpc_url = "http://127.0.0.1:8550"; // Reth node RPC

    let provider = ProviderBuilder::new()
        .wallet(wallet.clone())
        .on_http(rpc_url.parse().expect("Error parsing RPC URL"));

    let wallet_address: Address = signer.address();
    info!("Using wallet address: {:?}", wallet_address);

    let bob = address!("90F79bf6EB2c4f870365E785982E1f101E93b906");

    let client = HttpClientBuilder::default().build("http://127.0.0.1:3030")?;

    for _ in 0..3 {
        let nonce = provider.get_transaction_count(wallet_address).await?;
        info!(
            "Retrieved nonce for address {:?}: {}",
            wallet_address, nonce
        );

        let value: U256 = U256::from(rand::thread_rng().gen_range(1..100));
        info!("Generated random value for transaction: {}", value);

        let tx = TransactionRequest::default()
            .with_to(bob)
            .with_nonce(nonce)
            .with_chain_id(provider.get_chain_id().await?)
            .with_value(value)
            .with_gas_limit(21_000)
            .with_max_priority_fee_per_gas(1_000_000_000)
            .with_max_fee_per_gas(20_000_000_000);

        let tx_envelope = tx.build(&provider.wallet()).await?;
        info!("Signed transaction: {:?}", tx_envelope);

        match client
            .request::<String, [TxEnvelope; 1]>("eth_sendTransaction", [tx_envelope])
            .await
        {
            Ok(response) => {
                info!("Transaction sent to {}: response {}", bob, response);
            }
            Err(e) => {
                error!("Failed to send transaction: {:?}", e);
            }
        }
    }

    Ok(())
}
