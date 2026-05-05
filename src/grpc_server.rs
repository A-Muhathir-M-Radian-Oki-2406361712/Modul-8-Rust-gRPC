use tonic::{transport::Server, Request, Response, Status};

pub mod services {
    tonic::include_proto!("services");
}

use services::{payment_service_server::{PaymentService, PaymentServiceServer}, PaymentRequest, PaymentResponse};

#[derive(Default)]
pub struct MyPaymentService {}

#[tonic::async_trait]
impl PaymentService for MyPaymentService {
    async fn process_payment(&self, request: Request<PaymentRequest>) -> Result<Response<PaymentResponse>, Status> {
        let req = request.into_inner();
        println!("Received payment request: {:?}", req);

        // Here you would add your payment processing logic
        let response = PaymentResponse {
            success: true
        };

        Ok(Response::new(response))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let payment_service = MyPaymentService::default();

    println!("PaymentServiceServer listening on {}", addr);

    Server::builder()
        .add_service(PaymentServiceServer::new(payment_service))
        .serve(addr)
        .await?;

    Ok(())
}