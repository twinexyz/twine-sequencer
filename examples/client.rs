use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::core::client::ClientT;
use alloy::{
    network::{EthereumWallet, TransactionBuilder}, 
    node_bindings::Anvil, 
    primitives::U256, 
    providers::{Provider, ProviderBuilder, WalletProvider}, 
    rpc::types::TransactionRequest, 
    signers::local::PrivateKeySigner
};
use eyre::Result;
use rand::{Rng, seq::SliceRandom};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup Anvil test chain and provider
    let anvil = Anvil::new().block_time(1).try_spawn()?;
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer.clone());
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .on_http(anvil.endpoint_url());

    let accounts = provider.get_accounts().await?;
    let mut rng = rand::thread_rng();

    // Choose a random recipient
    let bob = accounts.choose(&mut rng).unwrap();  

    // Create the JSON-RPC HTTP client
    let client = HttpClientBuilder::default().build("http://127.0.0.1:3030")?;

    // Send three transactions in sequence
    for i in 0..3 {
        let nonce: u64 = rng.gen_range(0..100); 
        let value: U256 = U256::from(rng.gen_range(1..100)); 

        let tx = TransactionRequest::default()
            .with_to(*bob)  
            .with_nonce(nonce)
            .with_chain_id(provider.get_chain_id().await?)
            .with_value(value)
            .with_gas_limit(21_000)
            .with_max_priority_fee_per_gas(1_000_000_000)
            .with_max_fee_per_gas(20_000_000_000);

        // Build the transaction envelope and send the transaction
        let tx_envelope = tx.build(&provider.wallet()).await?;
        let response: String = client.request("eth_sendTransaction", [tx_envelope]).await?;

        // Print the result of each transaction
        println!("Transaction {} sent to address {}: response {}", i + 1, bob, response);
    }

    Ok(())
}
