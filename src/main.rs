mod streamer;
mod client;
//mod gui;

use streamer::ScreenStreamer;
use client::VideoPlayer;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err("Usage: <program> [server|client]".into());
    }

    let mode = &args[1];
    match mode.as_str() {
        "server" => {
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
        _ => return Err("Invalid mode. Use 'server' or 'client'".into()),
    }

    Ok(())
}

