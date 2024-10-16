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
#[tokio::main]
async fn main() -> Result<()> {
    let anvil = Anvil::new().block_time(1).try_spawn()?;
    let signer: PrivateKeySigner = anvil.keys()[0].clone().into();
    let wallet = EthereumWallet::from(signer.clone());
    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .on_http(anvil.endpoint_url());

    let accounts = provider.get_accounts().await?;

    let bob = accounts[1]; 
 

    let tx = TransactionRequest::default()
        .with_to(bob)
        .with_nonce(0) 
        .with_chain_id(provider.get_chain_id().await?)
        .with_value(U256::from(100))
        .with_gas_limit(21_000)
        .with_max_priority_fee_per_gas(1_000_000_000)
        .with_max_fee_per_gas(20_000_000_000);
       

    
    let tx_envelope = tx.build(&provider.wallet()).await?;

    let client = HttpClientBuilder::default().build("http://127.0.0.1:3030")?;

    let response: String = client.request("eth_sendTransaction", [tx_envelope]).await?;
    
    println!("Sent transaction: {}", response);
    

    Ok(())
}
