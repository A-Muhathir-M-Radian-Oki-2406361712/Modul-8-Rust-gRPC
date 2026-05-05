pub mod services {
    tonic::include_proto!("services");
}

use services::{PaymentRequest, transaction_service_client::TransactionServiceClient, TransactionRequest};
use services::{payment_service_client::PaymentServiceClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut payment_client = PaymentServiceClient::connect("http://[::1]:50051").await?;
    let mut transaction_client = TransactionServiceClient::connect("http://[::1]:50051").await?;

    let payment_request = tonic::Request::new(PaymentRequest {
        user_id: "user123".to_string(),
        amount: 100.0,
    });

    let payment_response = payment_client.process_payment(payment_request).await?;
    println!("Payment response: {:?}", payment_response.into_inner());

    let transaction_request = tonic::Request::new(TransactionRequest {
        user_id: "user123".to_string(),
    });

    let mut transaction_stream = transaction_client.get_transaction_history(transaction_request).await?.into_inner();

    while let Some(transaction) = transaction_stream.message().await? {
        println!("Received transaction: {:?}", transaction);
    }

    Ok(())
}