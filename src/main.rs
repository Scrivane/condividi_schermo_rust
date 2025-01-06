use std::error::Error;
use std::net::IpAddr;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use streamer::streamer::DimensionToCrop;

mod streamer;
mod connection;

mod gui;

use crate::gui::gui_main::run_iced;


use streamer::streamer::ScreenStreamer;
use streamer::client::StreamerClient;
use connection::client::DiscoveryClient;
use connection::server::DiscoveryServer;


#[cfg(target_os = "macos")]
#[link(name = "foundation", kind = "framework")]
extern "C" {
    fn CFRunLoopRun();
}

#[derive(PartialEq)] 
enum ControlMessage {
    Stop,
}

struct StreamerState {
    control_sender: mpsc::Sender<ControlMessage>,
    client_thread: thread::JoinHandle<()>,
    discovery_thread: thread::JoinHandle<()>,
    streamer_arc: Arc<Mutex<ScreenStreamer>>,
}


fn start_streamer(dimension: DimensionToCrop, num_monitor: usize) -> Result<StreamerState, Box<dyn Error>> {


    let (control_sender, control_receiver) = mpsc::channel();
    let (client_sender, client_receiver) = mpsc::channel();

    let streamer = ScreenStreamer::new(dimension, num_monitor).expect("errore creazione scren streamer");
    let streamer_arc = Arc::new(Mutex::new(streamer));

    let mut discovery_server = DiscoveryServer::new(client_sender);
    let discovery_thread = thread::spawn(move || {
        println!("Starting discovery server...");
        discovery_server.run_discovery_listener(control_receiver).expect("Failed to run discovery server");
        println!("finisce mai il server ????...");
    });    

    let streamer_arc_clone = Arc::clone(&streamer_arc);
    let client_thread = thread::spawn(move || {
        while let Ok(client_list) = client_receiver.recv() {
            let client_list_clone = client_list.clone();
            let streamer = streamer_arc_clone.lock().unwrap();
            streamer.update_clients(client_list);
            println!("Client list updated: {}", client_list_clone);
        }
    });

    {
        let mut streamer = streamer_arc.lock().unwrap();
        streamer.start().expect("error in starting the streamer");
        println!(
            "Streamer started\n\
            Press CTRL+C to stop the server\n\
            Press CTRL+P to pause the stream\n\
            Press CTRL+R to resume the stream"
        );
    }

    Ok(StreamerState {
        control_sender,
        client_thread,
        discovery_thread,
        streamer_arc,
    })
}


fn stop_streamer(state: StreamerState) -> Result<(), Box<dyn Error>> {
    // Send a stop message to the control thread
    state.control_sender.send(ControlMessage::Stop)?;

    // Wait for all threads to finish
    state.client_thread.join().expect("Client thread panicked");
    state.discovery_thread.join().expect("Discovery thread panicked");

    println!("Streamer stopped successfully.");

    Ok(())
}


fn start_client(ip_addr: IpAddr) -> Result<StreamerClient, Box<dyn Error>> {
    let discovery_client = DiscoveryClient::new()?;

    let (client_ip,client_port) = discovery_client.discover_server(ip_addr)?;

    let mut player = StreamerClient::new(client_ip.clone(),client_port)?;
    player.start_streaming()?;

    Ok(player)
}


fn stop_client(mut player:StreamerClient ) -> Result<(), Box<dyn Error>> {


    // andrebbe meso costrutto di sincronizzazione per evitare che cambi lo stato durante le successive istruzuini 
    match player.get_is_rec() {
        true => player.stop_recording()?,
        false => println!("It's not recording , we can end the stream "),
    } 
    
    player.stop_streaming();

    Ok(())

}


fn main()  {
   let res = run_iced();

   match res {
    Ok(_) => {
        println!("Application runned without error");
    },
    Err(e) => {
        println!("Error Launching Application: {}", e);
    },
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