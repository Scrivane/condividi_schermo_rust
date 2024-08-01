mod streamer;
mod client;

mod recorder;
//mod gui;

use streamer::ScreenStreamer;
use client::VideoPlayer;
use std::env;
use std::error::Error;
use std::process;
use std::thread;
use std::time::Duration;
fn main() -> Result<(), Box<dyn Error>> {

    /*
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Usage: <program> [server|client]".into());
    }

    let mode = &args[1];
    match mode.as_str() {
        "streamer" => {
            let mut streamer = ScreenStreamer::new()?;
            streamer.start()?;
            println!("Server started. Press Enter to stop...");
            let _ = std::io::stdin().read_line(&mut String::new());
            streamer.stop();
        }
        "client" => {
            let mut player = VideoPlayer::new()?;
            println!("Client started. Press Enter to stop...");
            let _ = std::io::stdin().read_line(&mut String::new());
            player.stop();
        }
        _ => return Err("Invalid mode. Use 'streamer' or 'client'".into()),
    }
    */

     let mut recorder = recorder::ScreenRecorder::new()?;
     recorder.start();
     println!("Registrazione avviata. Premere Ctrl+C per fermare.");

     ctrlc::set_handler(move || {
        recorder.stop();
        println!("Registrazione fermata.");
        recorder.stop();
        process::exit(0);
    }).expect("Errore interruzione programma");   
 
    // Mantiene il processo in esecuzione finché non viene premuto Ctrl+C
    loop {
        thread::sleep(Duration::from_secs(1));
    }




    Ok(())
}

