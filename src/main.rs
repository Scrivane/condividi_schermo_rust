use std::env;
use std::error::Error;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

mod streamer;
mod connection;
use std::io;
use std::net::SocketAddr;

use streamer::streamer::ScreenStreamer;
use streamer::client::StreamerClient;
use connection::client::DiscoveryClient;
use connection::server::DiscoveryServer;

use std::net::{ Ipv4Addr};
use socket2::{Socket, Domain, Type, Protocol,SockAddr};
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
                        KeyCode::Char('p') => sender.send(ControlMessage::Pause)?,
                        KeyCode::Char('r') => sender.send(ControlMessage::Resume)?,
                        KeyCode::Char('c') => {
                            sender.send(ControlMessage::Stop)?;
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

fn start_streamer() -> Result<(), Box<dyn Error>> {
    let (control_sender, control_receiver) = mpsc::channel();
    let (client_sender, client_receiver) = mpsc::channel();

    let streamer = ScreenStreamer::new()?;
    let streamer_arc = Arc::new(Mutex::new(streamer));

    let mut discovery_server = DiscoveryServer::new(client_sender);
    let discovery_thread = thread::spawn(move || {
        println!("Starting discovery server...");
        discovery_server.run_discovery_listener().expect("Failed to run discovery server");
    });

    // Gestisce i comandi di controllo in un thread separato
    let streamer_arc_clone = Arc::clone(&streamer_arc);
    let control_thread = thread::spawn(move || {
        while let Ok(message) = control_receiver.recv() {
            let mut streamer = streamer_arc_clone.lock().unwrap();
            match message {
                ControlMessage::Pause => streamer.pause(),
                ControlMessage::Resume => streamer.start().unwrap(),
                ControlMessage::Stop => {
                    streamer.stop();
                    break;
                }
            }
        }
    });

    // Gestisce l'aggiunta di nuovi client in un altro thread
    let streamer_arc_clone = Arc::clone(&streamer_arc);
    let client_thread = thread::spawn(move || {
        while let Ok(client_list) = client_receiver.recv() {
            let client_list_clone = client_list.clone();
            let  streamer = streamer_arc_clone.lock().unwrap();
            streamer.update_clients(client_list);
            println!("Client list update: {}", client_list_clone);
        }
    });

    // Avvia lo streamer e gestisce gli eventi della tastiera
    {
        let mut streamer = streamer_arc.lock().unwrap();
        streamer.start()?;
        println!(
            "Streamer started\n\
            Press CTRL+C to stop the server\n\
            Press CTRL+P to pause the stream\n\
            Press CTRL+R to resume the stream"
        );
    }

    handle_event(control_sender)?;

    // Aspetta la terminazione dei thread
    control_thread.join().expect("Control thread panicked");
    client_thread.join().expect("Client thread panicked");
    discovery_thread.join().expect("Discovery thread panicked");

    Ok(())
}

fn start_client() -> Result<(), Box<dyn Error>> {
    let discovery_client = DiscoveryClient::new()?;
    let (client_ip,client_port) = discovery_client.discover_server()?;
    //let client_port_clone = client_port.clone();
    

   // drop(discovery_client);  //fondamentale per disconnettere il socket e renderlo cosi possibile da usare per gstreamer
    let mut player = StreamerClient::new(client_ip.clone(),client_port)?;

    player.start()?;
    println!("Client started at port {}. Press Enter to stop...", &client_port);

    let _ = std::io::stdin().read_line(&mut String::new());
    player.stop();
    discovery_client.notify_disconnection()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Usage: <program> [streamer|client]".into());
    }

    match args[1].as_str() {
        "streamer" => start_streamer(),
        "client" => start_client(),
        _ => Err("Invalid mode. Use 'streamer' or 'client'".into()),
    }
}