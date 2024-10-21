use alloy::{
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

#[tokio::main]
async fn main() -> Result<()> {
    let signer: PrivateKeySigner =
        "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80"
            .trim_start_matches("0x")
            .parse()
            .expect("Error parsing private key");

    let wallet = EthereumWallet::from(signer.clone());
    let rpc_url = "http://127.0.0.1:8550"; //reth node rpc

    let provider = ProviderBuilder::new()
        .wallet(wallet.clone())
        .on_http(rpc_url.parse().expect("Error parsing RPC URL"));

    let wallet_address: Address = signer.address();

    let bob = address!("90F79bf6EB2c4f870365E785982E1f101E93b906");

    let client = HttpClientBuilder::default().build("http://127.0.0.1:3030")?;

    let nonce = provider.get_transaction_count(wallet_address).await?;

    let value: U256 = U256::from(rand::thread_rng().gen_range(1..100));

    let tx = TransactionRequest::default()
        .with_to(bob)
        .with_nonce(nonce)
        .with_chain_id(provider.get_chain_id().await?)
        .with_value(value)
        .with_gas_limit(21_000)
        .with_max_priority_fee_per_gas(1_000_000_000)
        .with_max_fee_per_gas(20_000_000_000);

    let tx_envelope = tx.build(&provider.wallet()).await?;
    println!("Signed transaction: {:?}", tx_envelope);

    let response: String = client.request("eth_sendTransaction", [tx_envelope]).await?;
    println!("Transaction sent to {}: response {}", bob, response);

    Ok(())
}
