use gst::prelude::*;
use gst::{Pipeline, State};
use std::env;
use std::fmt;

pub struct ScreenRecorder {
    pipeline: Option<Pipeline>,
    recording: bool,
}

//migliorare gestione errori
pub struct ServerError {
    message: String,
}

impl fmt::Debug for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ServerError: {}", self.message)
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ServerError: {}", self.message)
    }
}

impl std::error::Error for ServerError {}
impl ScreenRecorder {
    pub fn new() -> Result<Self, ServerError> {
        gst::init().unwrap();


        let pipeline = Pipeline::new();


        // Rileva il sistema operativo
        let os = env::consts::OS;


        // Crea l'elemento della sorgente video
        let videosrc = match os {
            "windows" => {
                gst::ElementFactory::make("d3d11screencapturesrc")
                    .property("display-id", &0)
                    .build()
                    .map_err(|_| ServerError{ message: "Failed to create d3d11screencapturesrc".to_string()})?
            }
            "macos" => {
                gst::ElementFactory::make("avfvideosrc")
                    .build()
                    .map_err(|_| ServerError{ message: "Failed to create avfvideosrc".to_string()})?
            }
            "linux" => {
                gst::ElementFactory::make("ximagesrc")
                    .build()
                    .map_err(|_| ServerError{ message: "Failed to create ximagesrc".to_string()})?
            }
            _ => {
                return Err(ServerError{ message: "OS non supportato".to_string()});
            }
        };


        let capsfilter = gst::ElementFactory::make("capsfilter")
            .property(
                "caps",
                gst::Caps::builder("video/x-raw")
                    .field("framerate", &gst::Fraction::new(30, 1))
                    .build(),
            )
            .build()
            .map_err(|_| ServerError { message: "Failed to create capsfilter".to_string() })?;


        let video_convert = gst::ElementFactory::make("videoconvert")
            .build()
            .map_err(|_| ServerError { message: "Failed to create videoconvert".to_string() })?;

        let video_encoder   = gst::ElementFactory::make("x264enc")
            .property_from_str("tune", "zerolatency")
            .build()
            .map_err(|_| ServerError { message: "Failed to create x264enc".to_string() })?;

        let flvmux = gst::ElementFactory::make("flvmux")
            .build()
            .map_err(|_| ServerError { message: "Failed to create flvmux".to_string() })?;


        let filesink = gst::ElementFactory::make("filesink")
            .property("location", &"video.flv")
            .build()
            .map_err(|_| ServerError { message: "Failed to create filesink".to_string() })?;


        // Aggiungi gli elementi alla pipeline
        pipeline.add_many(&[&videosrc, &capsfilter, &video_convert, &video_encoder, &flvmux, &filesink])
            .map_err(|_| ServerError {message: "Failed to add elements to pipeline".to_string()})?;


        // Collega gli elementi
        gst::Element::link_many(&[&videosrc, &capsfilter, &video_convert, &video_encoder, &flvmux, &filesink])
            .map_err(|_| ServerError {message: "Failed to link elements".to_string()})?;

        // Gestione degli eventi
        let bus = pipeline.bus().unwrap();
        let pipeline_clone = pipeline.clone();
        std::thread::spawn(move || {
            for msg in bus.iter_timed(gst::ClockTime::NONE) {
                match msg.view() {
                    gst::MessageView::Eos(..) => {
                        println!("End of stream");
                        pipeline_clone.set_state(State::Null).unwrap();
                        break;
                    }
                    gst::MessageView::Error(err) => {
                        println!(
                            "Error received from element {:?}: {:?}",
                            err.src().map(|s| s.path_string()),
                            err.error()
                        );
                        println!("Debugging information: {:?}", err.debug());
                        pipeline_clone.set_state(State::Null).unwrap();
                        break;
                    }
                    _ => (),
                }
            }
        });

        Ok(Self {
            pipeline: Some(pipeline),
            recording: false,
        })
    }



    pub fn start(&mut self) -> Result<(), String> {
        // Imposta la pipeline in stato di riproduzione
        let pipeline = self.pipeline.as_ref().unwrap();
        pipeline.set_state(State::Playing).map_err(|_| "Failed to set pipeline to Playing".to_string())?;
        self.recording = true;

        Ok(())
    }
    pub fn stop(&mut self) {
        if let Some(ref pipeline) = self.pipeline {
            pipeline.set_state(State::Null).map(|_| ());
        }
        self.pipeline = None;
        self.recording = false;
    }

}