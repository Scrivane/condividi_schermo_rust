mod streamer;
mod client;

mod recorder;

//mod gui

use streamer::ScreenStreamer;
use client::VideoPlayer;
use std::env;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
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

            let streamer_clone = Arc::clone(&streamer_arc);

            // Thread per ascoltare gli eventi della tastiera
            thread::spawn(move || {
                handle_event(sender.clone()).unwrap();
            });

            // Thread per gestire lo streaming
            let streaming_thread = {
                let streamer_arc = Arc::clone(&streamer_clone);
                thread::spawn(move || {
                    let mut streamer = streamer_arc.lock().unwrap();
                    streamer.start().expect("Failed to start the streamer");
                    println!("Server started...\n
                     Press CTRL+C to stop the server\n
                     Press CTRL+P to pause the stream\n
                     Press CTRL+R to resume the stream");


                    while let Ok(message) = receiver.recv() {
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
                })
            };

            streaming_thread.join().unwrap();
        }
        "client" => {
            let mut player = VideoPlayer::new()?;
            player.start()?;
            println!("Client started. Press Enter to stop...");
            let _ = std::io::stdin().read_line(&mut String::new());
            player.stop();
        }
        _ => return Err("Invalid mode. Use 'streamer' or 'client'".into()),
    }

    Ok(())
}