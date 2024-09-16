use std::net::{UdpSocket};
use std::sync::mpsc;
use std::error::Error;

pub struct DiscoveryServer {
    sender: mpsc::Sender<String>,
    clients: String,
}

impl DiscoveryServer {
    pub fn new(sender: mpsc::Sender<String>) -> Self {
        Self {
            sender,
            clients: String::new(),
        }
    }

    pub fn run_discovery_listener(&mut self) -> Result<(), Box<dyn Error>> {
        let socket = UdpSocket::bind("0.0.0.0:9000")?;

        println!("Discovery server is listening in {}",socket.local_addr().unwrap());

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
                //risponde al client dandogli l'indirizzo ip che verrà assegnato nel multiudp
                let response = format!("{}", src.ip().to_string());

                if let Err(e) = socket.send_to(response.as_bytes(), &src) {
                    println!("Failed to send response: {}", e);
                } else {
                    println!("Sent response '{}' to client {}", response, src);
                }


                // Aggiunge l'indirizzo del client alla lista formatta già correttamente per ScreenStreamer
                if !self.clients.is_empty() {
                    self.clients.push_str(&format!(",{}", src.to_string()));
                }
                else{
                    self.clients.push_str(&format!("{}", src.to_string()));
                }


                // Invia l'indirizzo del client al main tramite il canale

                //if let Err(e) = self.sender.send(src) {
                if let Err(e) = self.sender.send(self.clients.clone()) {
                    println!("Failed to send client list: {}", e);
                }
            }
            else if received_message.trim() == "DISCONNECT" {
                let clients_str: Vec<&str> = self.clients.split(',').filter(|&s| s != src.to_string()).collect();
                self.clients = clients_str.join(",");

                // Invia l'indirizzo del client al main tramite il canale
                if let Err(e) = self.sender.send(self.clients.clone()) {
                    println!("Failed to send client list: {}", e);
                }
            }
        }
    }
}
