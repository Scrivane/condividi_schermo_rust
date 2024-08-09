use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, ComboBoxText};
use std::sync::{Arc, Mutex};

use crate::recorder::ScreenRecorder;

pub fn build_ui(app: &Application) {
    // Crea una finestra
    let window = ApplicationWindow::new(app);
    window.set_title("Screen Recorder");
    window.set_default_size(350, 70);

    // Crea una ComboBox per selezionare lo schermo
    let combo = ComboBoxText::new();
    let screens = ScreenRecorder::get_screens();
    for (index, screen) in screens.iter().enumerate() {
        combo.append_text(&format!("Screen {}", index));
    }
    combo.set_active(Some(0));

    // Crea un pulsante per iniziare/fermare la registrazione
    let button = Button::with_label("Start Recording");

    // Layout
    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    vbox.pack_start(&combo, true, true, 0);
    vbox.pack_start(&button, true, true, 0);
    window.add(&vbox);

    // Pipeline e stato della registrazione
    let recorder = Arc::new(Mutex::new(ScreenRecorder::new()));

    // Clone per l'uso nel callback
    let recorder_clone = Arc::clone(&recorder);

    button.connect_clicked(move |button| {
        let mut recorder = recorder_clone.lock().unwrap();
        if recorder.is_recording() {
            recorder.stop();
            button.set_label("Start Recording");
        } else {
            let screen_index = combo.active().unwrap() as u32;
            recorder.start(screen_index);
            button.set_label("Stop Recording");
        }
    });

    window.show_all();
}