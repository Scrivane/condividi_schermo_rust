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
        let filename = desktop_path.join("recording.webm");
        println!("Filename {}",filename.to_str().unwrap());

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



        let videoconvert = gst::ElementFactory::make("videoconvert")
            .build()
            .map_err(|_| "Failed to create videoconvert".to_string())?;
        let vp8enc = gst::ElementFactory::make("vp8enc")
            .build()
            .map_err(|_| "Failed to create vp8enc".to_string())?;
        let webmmux = gst::ElementFactory::make("webmmux")
            .build()
            .map_err(|_| "Failed to create webmmux".to_string())?;
        let filesink = gst::ElementFactory::make("filesink")
            .property("location", filename.to_str().unwrap())
            .build()
            .map_err(|_| "Failed to create filesink".to_string())?;

        // Aggiungi gli elementi alla pipeline
        pipeline.add_many(&[&video_src, &videoconvert, &vp8enc, &webmmux, &filesink])
            .map_err(|_| "Failed to add elements to pipeline".to_string())?;

        // Collega gli elementi
        video_src.link(&videoconvert)
            .map_err(|_| "Failed to link video_src to videoconvert".to_string())?;
        videoconvert.link(&vp8enc)
            .map_err(|_| "Failed to link videoconvert to vp8enc".to_string())?;
        vp8enc.link(&webmmux)
            .map_err(|_| "Failed to link vp8enc to webmmux".to_string())?;

        // Aggiungi un elemento audio dummy per il muxer (opzionale)
        let audiotestsrc = gst::ElementFactory::make("audiotestsrc")
            .build()
            .map_err(|_| "Failed to create audiotestsrc".to_string())?;
        let audioconvert = gst::ElementFactory::make("audioconvert")
            .build()
            .map_err(|_| "Failed to create audioconvert".to_string())?;
        let audioresample = gst::ElementFactory::make("audioresample")
            .build()
            .map_err(|_| "Failed to create audioresample".to_string())?;
        let vorbisaenc = gst::ElementFactory::make("vorbisaenc")
            .build()
            .map_err(|_| "Failed to create vorbisaenc".to_string())?;

        pipeline.add_many(&[&audiotestsrc, &audioconvert, &audioresample, &vorbisaenc])
            .map_err(|_| "Failed to add audio elements to pipeline".to_string())?;

        audiotestsrc.link(&audioconvert)
            .map_err(|_| "Failed to link audiotestsrc to audioconvert".to_string())?;
        audioconvert.link(&audioresample)
            .map_err(|_| "Failed to link audioconvert to audioresample".to_string())?;
        audioresample.link(&vorbisaenc)
            .map_err(|_| "Failed to link audioresample to vorbisaenc".to_string())?;
        vorbisaenc.link(&webmmux)
            .map_err(|_| "Failed to link vorbisaenc to webmmux".to_string())?;

        webmmux.link(&filesink)
            .map_err(|_| "Failed to link webmmux to filesink".to_string())?;

        // Imposta la pipeline in stato di riproduzione
        pipeline.set_state(State::Playing)
            .map_err(|_| "Failed to set pipeline to Playing state".to_string())?;

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