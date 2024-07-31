use gst::prelude::*;
use gst::{Pipeline, State};
use std::env;
use std::path::PathBuf;
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

        /*
        // Ottieni il percorso del desktop
        let desktop_path = match os {
            "windows" => {
                let mut path = PathBuf::from(env::var("USERPROFILE").unwrap());
                path.push("Desktop");
                path
            }
            "macos" => {
                let mut path = PathBuf::from(env::var("HOME").unwrap());
                path.push("Desktop");
                path
            }
            "linux" => {
                let mut path = PathBuf::from(env::var("HOME").unwrap());
                path.push("Desktop");
                path
            }
            _ => return Err("OS non supportato".to_string()),
        };

        // Nome del file
        let filename = desktop_path.join("recording.mp4");
        println!("Filename {}",filename.to_str().unwrap());

        */

        // Crea l'elemento della sorgente video
        let video_src = match os {
            "windows" => {
                gst::ElementFactory::make("dshowvideosrc")
                    .property("device", &format!("screen://{}", screen_index))
                    .build()
                    .map_err(|_| "Failed to create dshowvideosrc".to_string())?
            }
            "macos" => {
                gst::ElementFactory::make("avfvideosrc")
                    .build()
                    .map_err(|_| "Failed to create avfvideosrc".to_string())?
            }
            "linux" => {
                gst::ElementFactory::make("ximagesrc")
                    .build()
                    .map_err(|_| "Failed to create ximagesrc".to_string())?
            }
            _ => {
                return Err("OS non supportato".to_string());
            }
        };

        let filename = PathBuf::from("output.mp4");

        let video_convert = gst::ElementFactory::make("videoconvert")
            .build()
            .map_err(|_| "Failed to create videoconvert".to_string())?;
        let video_encoder   = gst::ElementFactory::make("openh264enc")
            .build()
            .map_err(|_| "Failed to create avenc_mpeg4".to_string())?;
        let h264_parse  = gst::ElementFactory::make("h264parse")
            .build()
            .map_err(|_| "Failed to create h264parse".to_string())?;
        let mp4mux = gst::ElementFactory::make("mp4mux")
            .build()
            .map_err(|_| "Failed to create mp4mux".to_string())?;
        let filesink = gst::ElementFactory::make("filesink")
            .property("location", filename.to_str().unwrap())
            .build()
            .map_err(|_| "Failed to create filesink".to_string())?;

        // Aggiungi gli elementi alla pipeline
        pipeline.add_many(&[&video_src, &video_convert, &video_encoder, &h264_parse, &mp4mux, &filesink])
            .map_err(|_| "Failed to add elements to pipeline".to_string())?;

        // Collega gli elementi
        video_src.link(&video_convert)
            .map_err(|_| "Failed to link video_src to videoconvert".to_string())?;
        video_convert.link(&video_encoder)
            .map_err(|_| "Failed to link videoconvert to video_encoder".to_string())?;
        video_encoder.link(&h264_parse)
            .map_err(|_| "Failed to link h264_parse to video_encoder".to_string())?;
        h264_parse.link(&mp4mux)
            .map_err(|_| "Failed to link h264_parse to mp4mux".to_string())?;
        mp4mux.link(&filesink)
            .map_err(|_| "Failed to link mp4mux to filesink".to_string())?;


        // Imposta la pipeline in stato di riproduzione
        pipeline.set_state(State::Playing);


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