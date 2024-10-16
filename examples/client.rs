use jsonrpsee::http_client::HttpClientBuilder;
use alloy::primitives::U256;
use alloy::rpc::types::TransactionRequest;
use alloy::network::TransactionBuilder;
use jsonrpsee::core::client::ClientT;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = HttpClientBuilder::default().build("http://127.0.0.1:8545")?;

    let tx = TransactionRequest::default()
        .with_to("0x4B0897b0513fFdC404cD9487F0809E64C025c5b4".parse()?)
        .with_nonce(0)
        .with_chain_id(1)
        .with_value(U256::from(100))
        .with_gas_limit(21_000)
        .with_max_priority_fee_per_gas(1_000_000_000)  // 1 Gwei
        .with_max_fee_per_gas(20_000_000_000);         // 20 Gwei

    let response: String = client.request("eth_sendTransaction", [tx]).await?;
    println!("Transaction Hash: {}", response);

    Ok(())
}
