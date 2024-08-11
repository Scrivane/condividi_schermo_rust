use std::net::TcpStream;
use std::io::{Read, Write};
use std::fmt;

pub struct ServerClient {
    server_ip: String,
    server_port: u16,
}

pub struct ClientError {
    message: String,
}

impl fmt::Debug for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientError: {}", self.message)
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientError: {}", self.message)
    }
}

impl std::error::Error for ClientError {}

impl ServerClient {
    pub fn new(server_ip: &str, server_port: u16) -> Self {
        ServerClient {
            server_ip: server_ip.to_string(),
            server_port,
        }
    }

    pub fn connect(&self) -> Result<String, ClientError> {
        println!("Connecting to server at {}:{}", self.server_ip, self.server_port);
        let mut stream = TcpStream::connect((self.server_ip.as_str(), self.server_port))
            .map_err(|_| ClientError { message: "Failed to connect to server".to_string() })?;

        let mut ip_buffer = [0; 15];
        let n = stream.read(&mut ip_buffer)
            .map_err(|_| ClientError { message: "Failed to receive IP address from server".to_string() })?;
        let ip = String::from_utf8_lossy(&ip_buffer[..n]);

        if ip.contains("Connection refused") {
            return Err(ClientError { message: ip.to_string() });
        }
        println!("My IP: {}", ip);

        Ok(ip.to_string())
    }
}