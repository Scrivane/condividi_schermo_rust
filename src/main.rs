use std::env;
use std::error::Error;
use std::net::IpAddr;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use cfg_if::cfg_if;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use display_info::DisplayInfo;

mod streamer;
mod connection;
#[cfg(feature = "icedf")]
mod GUI_ADR;
#[cfg(feature = "icedf")]
use crate::GUI_ADR::gui_test::run_iced;


use streamer::streamer::ScreenStreamer;
use streamer::client::StreamerClient;
use connection::client::DiscoveryClient;
use connection::server::DiscoveryServer;


#[cfg(target_os = "macos")]
#[link(name = "foundation", kind = "framework")]
extern "C" {
    fn CFRunLoopRun();
}


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
#[cfg(feature = "icedf")]
fn start_streamer(num_monitor:usize) -> Result<(), Box<dyn Error>> { //mettere se si prova in modalità iced
    
  
    

   
    let (control_sender, control_receiver) = mpsc::channel();
    let (client_sender, client_receiver) = mpsc::channel();

    let streamer = ScreenStreamer::new(num_monitor)?;
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

#[cfg(not(feature = "icedf"))]
fn start_streamer() -> Result<(), Box<dyn Error>> { //mettere se si prova in modalità iced


    let num_monitor=select_monitor();
    
    

   
    let (control_sender, control_receiver) = mpsc::channel();
    let (client_sender, client_receiver) = mpsc::channel();

    let streamer = ScreenStreamer::new(num_monitor)?;
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


#[cfg(feature = "icedf")]
fn start_client(ip_addr: IpAddr) -> Result<(), Box<dyn Error>> {
    let discovery_client = DiscoveryClient::new()?;

    let (client_ip,client_port) = discovery_client.discover_server(ip_addr)?;
    //let client_port_clone = client_port.clone();
    

   // drop(discovery_client);  //fondamentale per disconnettere il socket e renderlo cosi possibile da usare per gstreamer
    let mut player = StreamerClient::new(client_ip.clone(),client_port)?;

    player.start_streaming()?;
    println!("Client started at port {}. Press Enter to stop...", &client_port);
    //player.start_recording()?;

    let _ = std::io::stdin().read_line(&mut String::new());

    //player.stop_recording()?;
    player.stop_streaming();

    Ok(())
}




#[cfg(not(feature = "icedf"))]
fn start_client() -> Result<(), Box<dyn Error>> {
    let discovery_client = DiscoveryClient::new()?;
    let (client_ip,client_port) = discovery_client.discover_server()?;
    //let client_port_clone = client_port.clone();
    

   // drop(discovery_client);  //fondamentale per disconnettere il socket e renderlo cosi possibile da usare per gstreamer
    let mut player = StreamerClient::new(client_ip.clone(),client_port)?;

    player.start_streaming()?;
    println!("Client started at port {}. Press Enter to stop...", &client_port);
    //player.start_recording()?;

    let _ = std::io::stdin().read_line(&mut String::new());

    //player.stop_recording()?;
    player.stop_streaming();

    Ok(())
}


fn select_monitor() -> usize {
    // Inizializza la finestra UI lampo
    let display_infos = DisplayInfo::all().unwrap();
    let mut monitor_ids = Vec::new();
    let mut id = 0;

    for display_info in display_infos {
        println!("Name : {}, number {}", display_info.name, id);
        monitor_ids.push(id);
        id += 1;
    }

    let mut selected_monitor = 0;
    loop {
        println!("Select the monitor to stream (0, 1, 2, ...): ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).expect("Failed to read line");
        match input.trim().parse() {
            Ok(num) if num < monitor_ids.len() => {
                selected_monitor = num;
                break;
            }
            _ => {
                println!("Invalid input. Please enter a valid monitor number.");
            }
        }
    }
    selected_monitor
}
#[cfg(feature = "icedf")]
fn main()  {
    run_iced();
}

#[cfg(not(feature = "icedf"))]
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

/// macOS ha bisogno di un run loop per aprire finestre e utilizzare OpenGL.
#[cfg(target_os = "macos")]
pub fn run<F: FnOnce() + Send + 'static>(main: F) {
    std::thread::spawn(main);
    unsafe {
        CFRunLoopRun();
    }
}

#[cfg(not(target_os = "macos"))]
pub fn run<F: FnOnce() + Send + 'static>(main: F) {
    main();
}