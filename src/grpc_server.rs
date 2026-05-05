use tonic::{transport::Server, Request, Response, Status};

pub mod services {
    tonic::include_proto!("services");
}

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio::sync::mpsc::{Receiver, Sender};

use services::{payment_service_server::{PaymentService, PaymentServiceServer}, PaymentRequest, PaymentResponse,
    transaction_service_server::{TransactionService, TransactionServiceServer}, TransactionRequest, TransactionResponse,
    chat_service_server::{ChatService, ChatServiceServer}, ChatMessage};

#[derive(Default)]
pub struct MyPaymentService {}

#[derive(Default)]
pub struct MyTransactionService {}

#[derive(Default)]
pub struct MyChatService {}

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

#[tonic::async_trait]
impl TransactionService for MyTransactionService {
    type GetTransactionHistoryStream = ReceiverStream<Result<TransactionResponse, Status>>;

    async fn get_transaction_history(&self, request: Request<TransactionRequest>) -> Result<Response<Self::GetTransactionHistoryStream>, Status> {
        let req = request.into_inner();
        println!("Received transaction request: {:?}", req);

        let (tx, rx): (Sender<Result<TransactionResponse, Status>>, Receiver<Result<TransactionResponse, Status>>) = mpsc::channel(4);

        // Simulate streaming transactions
        tokio::spawn(async move {
            for i in 0..16 {
                let response = TransactionResponse {
                    transaction_id: format!("trans_{}", i),
                    status: "Completed".to_string(),
                    amount: 100.0 + i as f64,
                    timestamp: (1632_000_000 + i * 60).to_string(), // Simulated timestamp
                };
                if let Err(e) = tx.send(Ok(response)).await {
                    eprintln!("Failed to send transaction response: {}", e);
                    return;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
#[tonic::async_trait]
impl ChatService for MyChatService {
    type ChatStream = ReceiverStream<Result<ChatMessage, Status>>;

    async fn chat(
        &self,
        request: Request<tonic::Streaming<ChatMessage>>,
    ) -> Result<Response<Self::ChatStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = mpsc::channel(10);

        tokio::spawn(async move {
            while let Some(message) = stream.message().await.unwrap_or_else(|_| None) {
                println!("Received message: {:?}", message);
                let reply = ChatMessage {
                    user_id: message.user_id.clone(),
                    message: format!("Terima kasih telah melakukan chat kepada CS virtual, Pesan anda akan dibalas pada jam kerja. pesan: {}", message.message),
                };

                tx.send(Ok(reply)).await.unwrap_or_else(|_| {});
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let payment_service = MyPaymentService::default();
    let transaction_service = MyTransactionService::default();
    let chat_service = MyChatService::default();

    println!("TransactionServiceServer listening on {}", addr);

    Server::builder()
        .add_service(PaymentServiceServer::new(payment_service))
        .add_service(TransactionServiceServer::new(transaction_service))
        .add_service(ChatServiceServer::new(chat_service))
        .serve(addr)
        .await?;

    Ok(())
}
