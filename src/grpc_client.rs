pub mod services {
    tonic::include_proto!("services");
}

use services::payment_service_client::PaymentServiceClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = PaymentServiceClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(services::PaymentRequest {
        user_id: "user123".to_string(),
        amount: 100.0,
    });

    let response = client.process_payment(request).await?;
    println!("Response from server: {:?}", response.into_inner());

    Ok(())
}