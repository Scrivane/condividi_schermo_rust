use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::streamer::streamer::ScreenStreamer;

pub struct Server {
    streamer: Arc<Mutex<ScreenStreamer>>,
    clients: Arc<Mutex<Vec<String>>>,
    max_clients: usize,
}

impl Server {
    pub fn new(streamer: Arc<Mutex<ScreenStreamer>>, max_clients: usize) -> Self {
        Server {
            streamer,
            clients: Arc::new(Mutex::new(Vec::new())),
            max_clients,
        }
    }

    pub fn start(&self) {
        let listener = TcpListener::bind("0.0.0.0:12345").unwrap();

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut clients_guard = self.clients.lock().unwrap();
                    if clients_guard.len() >= self.max_clients {
                        let _ = stream.write(b"Connection refused: Server is full.\n");
                        continue;
                    }

                   //generate ip for each client
                    let ip = format!("192.168.1.{}", clients_guard.len() + 2);

                    // add ip to list of clients
                    clients_guard.push(ip.clone());

                    stream.write(ip.as_bytes()).unwrap();

                    let streamer_clone = Arc::clone(&self.streamer);
                    let clients_clone = Arc::clone(&self.clients);
                    let ip_clone = ip.clone();

                    thread::spawn(move || {
                        Self::handle_client(stream, streamer_clone, clients_clone, ip_clone);
                    });
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
    }

    fn handle_client(mut stream: TcpStream, streamer: Arc<Mutex<ScreenStreamer>>, clients: Arc<Mutex<Vec<String>>>, ip: String) {
        let mut buffer = [0; 512];
        while match stream.read(&mut buffer) {
            Ok(size) => {
                if size == 0 {
                    // Rimuovi l'IP dalla lista dei client
                    streamer.lock().unwrap().remove_client(&ip).unwrap();
                    clients.lock().unwrap().retain(|x| x != &ip);
                    return;
                }
                true
            }
            Err(_) => {
                eprintln!("An error occurred, terminating connection with {}", ip);
                false
            }
        } {}
    }

    pub fn list_clients(&self) -> Vec<String> {
        self.clients.lock().unwrap().clone()
    }
}