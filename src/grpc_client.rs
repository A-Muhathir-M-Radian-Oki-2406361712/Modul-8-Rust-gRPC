pub mod services {
    tonic::include_proto!("services");
}

use services::{transaction_service_client::TransactionServiceClient, TransactionRequest};
use services::{payment_service_client::PaymentServiceClient, PaymentRequest};
use services::{chat_service_client::ChatServiceClient, ChatMessage};
use tonic::transport::channel;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::io::{self, AsyncBufReadExt};
use tokio_stream::wrappers::ReceiverStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut payment_client = PaymentServiceClient::connect("http://[::1]:50051").await?;
    let mut transaction_client = TransactionServiceClient::connect("http://[::1]:50051").await?;
    
    let channel = channel::Channel::from_static("http://[::1]:50051").connect().await?;
    let mut chat_client = ChatServiceClient::new(channel);
    let (tx, rx): (Sender<ChatMessage>, Receiver<ChatMessage>) = mpsc::channel(8);

    // 1. Simpan handle task ini agar bisa kita tunggu di akhir program
    let stdin_handle = tokio::spawn(async move {
        let stdin = io::stdin();
        let mut reader = io::BufReader::new(stdin).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            if line.trim().is_empty() {
                continue;
            }

            let chat_message = ChatMessage {
                user_id: "user_123".to_string(),
                message: line,
            };
            if let Err(e) = tx.send(chat_message).await {
                eprintln!("Failed to send chat message: {}", e);
                break; // Keluar dari loop jika channel tx terputus
            };
        }
    });
    
    let request_stream = ReceiverStream::new(rx);
    let response = chat_client.chat(tonic::Request::new(request_stream)).await?;

    // 2. Ekstrak stream balasan dari server dan baca di background
    let mut incoming_chat = response.into_inner();
    tokio::spawn(async move {
        while let Ok(Some(msg)) = incoming_chat.message().await {
            println!("\n[Chat from Server]: {}", msg.message);
        }
    });

    // --- (Proses Payment) ---
    let payment_request = tonic::Request::new(PaymentRequest {
        user_id: "user123".to_string(),
        amount: 100.0,
    });
    let payment_response = payment_client.process_payment(payment_request).await?;
    println!("Payment response: {:?}", payment_response.into_inner());

    // --- (Proses Transaction) ---
    let transaction_request = tonic::Request::new(TransactionRequest {
        user_id: "user123".to_string(),
    });
    let mut transaction_stream = transaction_client.get_transaction_history(transaction_request).await?.into_inner();

    while let Some(transaction) = transaction_stream.message().await? {
        println!("Received transaction: {:?}", transaction);
    }
    
    println!("Transactions done. You can continue chatting...");

    // 3. Tahan fungsi main agar tidak langsung exit. 
    // Program baru akan berhenti jika kamu menghentikan input terminal (misal: Ctrl+C)
    let _ = stdin_handle.await;

    Ok(())
}