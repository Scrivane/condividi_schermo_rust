use tokio_tungstenite::connect_async;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use tokio::net::UdpSocket;
use tokio::time::Duration;


pub async fn request_connection() -> Result<(), Box<dyn std::error::Error>> {
    let url = "ws://127.0.0.1:8080";
    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    ws_stream.send("request_ip".into()).await?;

    if let Some(Ok(msg)) = ws_stream.next().await {
        if msg.is_text() {
            let assigned_ip = msg.to_text().unwrap();
            println!("Assigned IP: {}", assigned_ip);

            let socket = UdpSocket::bind("0.0.0.0:0").await?;
            socket.connect(format!("{}:5000", assigned_ip)).await?;

        }
    }

    Ok(())
}