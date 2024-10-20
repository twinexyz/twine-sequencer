use alloy::{
    network::{EthereumWallet, TransactionBuilder}, 
    node_bindings::Anvil, 
    primitives::{U256, B256}, 
    providers::{Provider, ProviderBuilder, WalletProvider}, 
    rpc::types::TransactionRequest, 
    signers::local::PrivateKeySigner
};
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::core::client::ClientT;
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

    // Create the JSON-RPC HTTP client to communicate with your RPC server
    let client = HttpClientBuilder::default().build("http://127.0.0.1:3030")?;

    // Send three signed transactions in sequence
    for i in 0..3 {
        // Generate a random nonce and value
        let nonce: u64 = rng.gen_range(0..100); 
        let value: U256 = U256::from(rng.gen_range(1..100)); 

        // Build the transaction request
        let tx = TransactionRequest::default()
            .with_to(*bob)  
            .with_nonce(nonce)
            .with_chain_id(provider.get_chain_id().await?)
            .with_value(value)
            .with_gas_limit(21_000)
            .with_max_priority_fee_per_gas(1_000_000_000)
            .with_max_fee_per_gas(20_000_000_000);

        // Build and sign the transaction envelope
        let tx_envelope = tx.build(&provider.wallet()).await?;
        println!("{:?}",tx_envelope);

        // Extract the signature from the transaction envelope
        let signature = tx_envelope.signature_hash().clone(); // Assuming there's a method to get the signature
        println!("Signature for transaction {}: {:?}", i + 1, signature);

        // Send the transaction to the RPC server using eth_sendTransaction
        let response: String = client.request("eth_sendTransaction", [tx_envelope]).await?;

        // Print the result of each transaction
        println!("Transaction {} sent to address {}: response {}", i + 1, bob, response);
    }

    Ok(())
}


// //! Example showing how to send an [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559) transaction.

// use alloy::{
//     network::TransactionBuilder,
//     primitives::U256,
//     providers::{Provider, ProviderBuilder},
//     rpc::types::TransactionRequest,
// };
// use eyre::Result;

// #[tokio::main]
// async fn main() -> Result<()> {
//     // Spin up a local Anvil node.
//     // Ensure `anvil` is available in $PATH.
//     let provider = ProviderBuilder::new().on_anvil_with_wallet();

//     // Create two users, Alice and Bob.
//     let accounts = provider.get_accounts().await?;
//     let alice = accounts[0];
//     let bob = accounts[1];

//     // Build a transaction to send 100 wei from Alice to Bob.
//     // The `from` field is automatically filled to the first signer's address (Alice).
//     let tx = TransactionRequest::default()
//         .with_to(bob)
//         .with_nonce(0)
//         .with_chain_id(provider.get_chain_id().await?)
//         .with_value(U256::from(100))
//         .with_gas_limit(21_000)
//         .with_max_priority_fee_per_gas(1_000_000_000)
//         .with_max_fee_per_gas(20_000_000_000);

//     // Send the transaction and wait for the broadcast.
//     let pending_tx = provider.send_transaction(tx).await?;

//     println!("Pending transaction... {}", pending_tx.tx_hash());

//     // Wait for the transaction to be included and get the receipt.
//     let receipt = pending_tx.get_receipt().await?;

//     println!(
//         "Transaction included in block {}",
//         receipt.block_number.expect("Failed to get block number")
//     );

//     assert_eq!(receipt.from, alice);
//     assert_eq!(receipt.to, Some(bob));

//     Ok(())
// }