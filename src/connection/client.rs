use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::process::Command;
use crate::models::{ClientRequest, ServerResponse};

pub async fn request_streaming_access(server_addr: &str, client_id: &str) -> Option<ServerResponse> {
    let request = ClientRequest {
        id: client_id.to_string(),
    };

    let mut socket = TcpStream::connect(server_addr).await.unwrap();
    let request_data = serde_json::to_vec(&request).unwrap();
    socket.write_all(&request_data).await.unwrap();

    let mut buf = [0; 1024];
    let n = socket.read(&mut buf).await.unwrap();

    let response: ServerResponse = serde_json::from_slice(&buf[..n]).unwrap();
    println!("Received IP: {}, Port: {}", response.ip, response.port);

    Some(response)
}