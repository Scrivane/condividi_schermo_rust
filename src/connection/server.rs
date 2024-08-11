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
        let listener = TcpListener::bind("127.0.0.1:9000").unwrap();
        println!("Server listening on port 9000");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());

                    // Genera l'IP per il nuovo client
                    let ip = format!("192.168.1.{}", self.clients.lock().unwrap().len() + 2);
                    let ip_clone = ip.clone();

                    stream.write(ip_clone.as_bytes()).unwrap();

                    self.streamer.lock().unwrap().add_client(ip).expect("TODO: panic message");

                    self.clients.lock().unwrap().push(ip_clone);


                    // Avvia un thread per gestire il client
                    /*
                    thread::spawn(move || {
                        Self::handle_client(stream, streamer_clone, self.clients.clone(), ip_clone);
                    });


                     */
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
                    if let Err(e) = streamer.lock().unwrap().remove_client(&ip) {
                        eprintln!("Failed to remove client from streamer: {}", e);
                    }
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
        // Rimuove il client dallo ScreenStreamer e dalla lista dei client
        if let Err(e) = streamer.lock().unwrap().remove_client(&ip) {
            eprintln!("Failed to remove client from streamer: {}", e);
        }
        clients.lock().unwrap().retain(|x| x != &ip);
    }

    pub fn list_clients(&self) -> Vec<String> {
        self.clients.lock().unwrap().clone()
    }
}