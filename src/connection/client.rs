use std::net::UdpSocket;
use std::io;

pub struct DiscoveryClient{
    local_port: u16,
    socket: UdpSocket,
}

impl DiscoveryClient {
    pub fn new() -> Result<Self, io::Error> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        let local_addr = socket.local_addr()?;
        let local_port = local_addr.port();

        Ok(DiscoveryClient { socket, local_port })
    }

    pub fn discover_server(&self) -> Result<(String,i32), io::Error> {
        let server_addr = "255.255.255.255:9000"; // Broadcast al server di scoperta

        println!("Sending DISCOVERY message to {}", server_addr);

        self.socket.set_broadcast(true)?;

        let discovery_message = "DISCOVERY";
        self.socket.send_to(discovery_message.as_bytes(), server_addr)?;
        println!("Sent DISCOVERY message with local port: {}", self.local_port);


        let mut buf = [0; 1024];
        match self.socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                let server_response = String::from_utf8_lossy(&buf[..amt]).to_string();
                println!("Received response: {} from {}", server_response,src);
                let ip = src.to_string().split(":").next().unwrap().to_string(); //takes only the ip address
                Ok((ip, self.local_port as i32))
            },
            Err(e) => {
                println!("Failed to receive response: {}", e);
                Err(e)
            }
        }
    }

    pub fn notify_disconnection(&self) -> Result<(), io::Error> {
        let server_addr = "255.255.255.255:9000"; // L'indirizzo del server

        println!("Sending DISCONNECT message to {}", server_addr);

        let disconnect_message = format!("DISCONNECT:{}", self.local_port);
        self.socket.send_to(disconnect_message.as_bytes(), server_addr)?;
        println!("Sent DISCONNECT message with local port: {}", self.local_port);

        Ok(())
    }
}
