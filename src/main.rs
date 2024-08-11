mod streamer;
mod connection;

use streamer::streamer::ScreenStreamer;
use streamer::client::StreamerClient;
use connection::client::DiscoveryClient;
use connection::server::DiscoveryServer;

use std::env;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

enum ControlMessage {
    Pause,
    Resume,
    Stop,
}

fn handle_event(sender: mpsc::Sender<ControlMessage>) -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    loop {
        if event::poll(std::time::Duration::from_millis(500))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    match key_event.code {
                        KeyCode::Char('p') => {
                            if sender.send(ControlMessage::Pause).is_err() {
                                println!("Failed to send Pause message");
                            }
                        }
                        KeyCode::Char('r') => {
                            if sender.send(ControlMessage::Resume).is_err() {
                                println!("Failed to send Resume message");
                            }
                        }
                        KeyCode::Char('c') => {
                            if sender.send(ControlMessage::Stop).is_err() {
                                println!("Failed to send Stop message");
                            }
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    disable_raw_mode()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Usage: <program> [streamer|client]".into());
    }

    let mode = &args[1];
    match mode.as_str() {
        "streamer" => {
            let (sender, receiver) = mpsc::channel();
            let streamer = ScreenStreamer::new()?;
            let streamer_arc = Arc::new(Mutex::new(streamer));
            let streamer_arc_clone = Arc::clone(&streamer_arc);

            // Avvia il server di scoperta in un thread separato
            let discovery_server = DiscoveryServer::new(streamer_arc_clone);
            let server_thread = thread::spawn(move || {
                println!("Starting discovery server...");
                discovery_server.run_discovery_listener().expect("Failed to run discovery server");
            });

            // Avvia lo streaming dopo che il server è avviato
            {
                let mut streamer = streamer_arc.lock().unwrap();
                streamer.start().expect("Failed to start the streamer");
                println!(
                    "Streamer started\n\
                     Press CTRL+C to stop the server\n\
                     Press CTRL+P to pause the stream\n\
                     Press CTRL+R to resume the stream"
                );

                // Gestisci gli eventi della tastiera
                handle_event(sender)?;
            }

            // Gestisci i comandi di controllo
            while let Ok(message) = receiver.recv() {
                let mut streamer = streamer_arc.lock().unwrap();
                match message {
                    ControlMessage::Pause => {
                        streamer.pause();
                    }
                    ControlMessage::Resume => {
                        streamer.start().unwrap();
                    }
                    ControlMessage::Stop => {
                        streamer.stop();
                        break;
                    }
                }
            }


            server_thread.join().expect("Server thread panicked");
        }
        "client" => {
            // Client per scoprire il server e ottenere l'IP
            let discovery_client = DiscoveryClient::new();
            let server_ip = discovery_client.discover_server()?; // Scopre il server e ottiene l'IP

            // Client dello streamer per avviare lo streaming
            let mut player = StreamerClient::new(server_ip.clone())?;

            player.start()?;
            println!("Client started streaming from server at {}. Press Enter to stop...", server_ip);
            let _ = std::io::stdin().read_line(&mut String::new());
            player.stop();
        }
        _ => return Err("Invalid mode. Use 'streamer' or 'client'".into()),
    }

    Ok(())
}
