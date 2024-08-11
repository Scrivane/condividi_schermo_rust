use std::net::UdpSocket;
use std::io;

pub struct DiscoveryClient;

impl DiscoveryClient {
    pub fn new() -> Self {
        DiscoveryClient {}
    }

    pub fn discover_server(&self) -> Result<String, io::Error> {
        let socket = UdpSocket::bind("0.0.0.0:0")?; // Bind alla porta scelta
        let server_addr = "255.255.255.255:9000"; // Broadcast al server di scoperta

        println!("Sending DISCOVERY message to {}", server_addr);

        socket.set_broadcast(true)?;
        socket.send_to(b"DISCOVERY", server_addr)?;
        println!("Sent DISCOVERY message");

        let mut buf = [0; 1024];
        match socket.recv_from(&mut buf) {
            Ok((amt, _)) => {
                let server_response = String::from_utf8_lossy(&buf[..amt]).to_string();
                println!("Received response: {}", server_response);
                Ok(server_response)
            },
            Err(e) => {
                println!("Failed to receive response: {}", e);
                Err(e)
            }
        }
    }
}
