mod recorder;
//mod gui;

use std::io::{self, Write};
use std::process;
use std::thread;
use std::time::Duration;

fn main() {
    let screen = read_input("Specifica il numero dello schermo da registrare (es. 0): ");
    let output = read_input("Specifica il percorso del file di output (es. output.mp4): ");

    let screen: u32 = screen.trim().parse().unwrap_or_else(|e| {
        eprintln!("Errore nel parsing del numero dello schermo: {}", e);
        process::exit(1);
    });

    let output = output.trim();

    let mut recorder = recorder::ScreenRecorder::new();
    recorder.start(screen);

    println!("Registrazione avviata. Premere Ctrl+C per fermare.");

    ctrlc::set_handler(move || {
        recorder.stop();
        println!("Registrazione fermata.");
        process::exit(0);
    })
        .expect("Errore nell'impostazione dell'handler Ctrl+C");

    // Mantiene il processo in esecuzione finché non viene premuto Ctrl+C
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}

fn read_input(prompt: &str) -> String {
    let mut input = String::new();
    print!("{}", prompt);
    io::stdout().flush().expect("Errore nel flush dell'output");
    io::stdin().read_line(&mut input).expect("Errore nella lettura dell'input");
    input
}