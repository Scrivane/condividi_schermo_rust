use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::models::{ClientRequest, ServerResponse};

pub struct ServerState {
    base_ip: String,
    base_port: u16,
    max_clients: u16,
    assigned_clients: HashMap<String, (String, u16)>,
}

impl ServerState {
    pub fn new(base_ip: &str, base_port: u16, max_clients: u16) -> Self {
        ServerState {
            base_ip: base_ip.to_string(),
            base_port,
            max_clients,
            assigned_clients: HashMap::new(),
        }
    }

    pub fn get_next_ip_port(&mut self, client_id: &str) -> Option<(String, u16)> {
        if self.assigned_clients.len() as u16 >= self.max_clients {
            return None;
        }
        let client_num = self.assigned_clients.len() as u16 + 1;
        let ip = format!("{}.{}", self.base_ip, 100 + client_num);
        let port = self.base_port + client_num;
        self.assigned_clients.insert(client_id.to_string(), (ip.clone(), port));
        Some((ip, port))
    }
}

pub async fn handle_client(mut socket: TcpStream, state: Arc<Mutex<ServerState>>) {
    let mut buf = [0; 1024];

    if let Ok(n) = socket.read(&mut buf).await {
        let request: ClientRequest = serde_json::from_slice(&buf[..n]).unwrap();
        println!("Request received from: {:?}", request);

        let mut state = state.lock().unwrap();
        if let Some((ip, port)) = state.get_next_ip_port(&request.id) {
            let response = ServerResponse { ip, port };
            let response_data = serde_json::to_vec(&response).unwrap();
            socket.write_all(&response_data).await.unwrap();
            println!("Assigned IP: {}, Port: {} a {}", ip, port, request.id);
        } else {
            let error_message = b"Error: max clients reached";
            socket.write_all(error_message).await.unwrap();
            println!("Max clients reached");
        }
    }
}

pub async fn run_server(state: Arc<Mutex<ServerState>>, addr: &str) {
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("Server running on {}", addr);

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let state = state.clone();

        tokio::spawn(async move {
            handle_client(socket, state).await;
        });
    }
}