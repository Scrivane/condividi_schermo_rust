use std::collections::HashMap;
use std::net::{UdpSocket, SocketAddr};
use std::sync::{Arc, Mutex};
use std::error::Error;
use crate::streamer::streamer::ScreenStreamer;

pub struct DiscoveryServer {
    clients: Arc<Mutex<HashMap<SocketAddr, String>>>,
    streamer: Option<ScreenStreamer>
}

impl DiscoveryServer {
    pub fn new( streamer: ScreenStreamer) -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            streamer: Some(streamer),
        }
    }

    pub fn run_discovery_listener(&self) -> Result<(), Box<dyn Error>> {
        let socket = UdpSocket::bind("0.0.0.0:9000")?;
        let clients = Arc::clone(&self.clients);

        println!("Discovery server is listening on port 9000");

        loop {
            let mut buf = [0; 1024];
            let (amt, src) = match socket.recv_from(&mut buf) {
                Ok(result) => result,
                Err(e) => {
                    println!("Error receiving data: {}", e);
                    continue;
                }
            };

            let received_message = String::from_utf8_lossy(&buf[..amt]);
            println!("Received message: '{}' from {}", received_message, src);

            if received_message.trim() == "DISCOVERY" {
                let server_ip = "127.0.0.1"; // IP del server di streaming
                let server_port = 5000; // Porta del server di streaming
                let response = format!("{}:{}", server_ip, server_port);

                if let Err(e) = socket.send_to(response.as_bytes(), &src) {
                    println!("Failed to send response: {}", e);
                } else {
                    println!("Sent response '{}' to client {}", response, src);
                }

                // Salva l'IP e la porta del client
                let mut clients = clients.lock().unwrap();
                clients.insert(src, response);

                // Usa `self.streamer` direttamente
                if let Some(ref mut streamer) = self.streamer {
                    streamer.add_client(src.to_string());
                    println!("Client {} added to list", src);
                }
            }
        }
    }

    pub fn get_clients(&self) -> Arc<Mutex<HashMap<SocketAddr, String>>> {
        Arc::clone(&self.clients)
    }
}
