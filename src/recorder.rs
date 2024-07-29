use gst::prelude::*;
use gst::{Element,Pipeline, State, ErrorMessage};
use std::env;
pub struct ScreenRecorder {
    pipeline: Option<Pipeline>,
    recording: bool,
}

impl ScreenRecorder {
    pub fn new() -> Self {
        gst::init().unwrap();
        ScreenRecorder {
            pipeline: None,
            recording: false,
        }
    }

    pub fn get_screens() -> Vec<String> {
        // Legge gli schermi disponibili
        vec!["Screen 0".to_string(), "Screen 1".to_string()]
    }

    pub fn start(&mut self, screen_index: u32) -> Result<(), String> {
        let pipeline = Pipeline::new();

        // Rileva il sistema operativo
        let os = env::consts::OS;

        // Crea l'elemento della sorgente video
        let video_src = match os {
            "windows" => {
                let src = gst::ElementFactory::make("dshowvideosrc")
                    .property("device", &format!("screen://{}", screen_index))
                    .build()
                    .unwrap();
                src
            }
            "macos" => {
                let src = gst::ElementFactory::make("avfvideosrc")
                    .build()
                    .unwrap();

                src
            }
            "linux" => {
                let src = gst::ElementFactory::make("ximagesrc")
                    .build()
                    .unwrap();
                // Se necessario, imposta altre proprietà specifiche per ximagesrc
                src
            }
            _ => {
                return Err("OS non supportato".to_string());
            }
        };

        let videoconvert = gst::ElementFactory::make("videoconvert").build().unwrap();
        let x264enc = gst::ElementFactory::make("x264enc").build().unwrap();
        let mp4mux = gst::ElementFactory::make("mp4mux").build().unwrap();
        let filesink = gst::ElementFactory::make("filesink")
            .property("location", "recording.mp4")
            .build()
            .unwrap();

        // Aggiungi gli elementi alla pipeline
        pipeline.add_many([&video_src, &videoconvert, &x264enc, &mp4mux, &filesink]).unwrap();

        // Collega gli elementi
        video_src.link(&videoconvert).expect("Impossibile collegare ximagesrc a videoconvert");
        videoconvert.link(&x264enc).expect("Impossibile collegare videoconvert a vp8enc");
        x264enc.link(&mp4mux).expect("Impossibile collegare x264enc a mp4mux");
        mp4mux.link(&filesink).expect("Impossibile collegare mp4mux a filesink");

        // Imposta la pipeline in stato di riproduzione
        pipeline.set_state(State::Playing).expect("Impossibile avviare la pipeline");

        self.pipeline = Some(pipeline);
        self.recording = true;

        Ok(())
    }
    pub fn stop(&mut self) {
        if let Some(ref pipeline) = self.pipeline {
            pipeline.set_state(State::Null).unwrap();
        }
        self.pipeline = None;
        self.recording = false;
    }

    pub fn is_recording(&self) -> bool {
        self.recording
    }
}