use gst::prelude::*;
use gst::{Pipeline, State};
use std::fmt;

pub struct VideoPlayer {
    pipeline: Option<Pipeline>,
}

pub struct ClientError {
    message: String,
}

impl fmt::Debug for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientError: {}", self.message)
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ClientError: {}", self.message)
    }
}

impl std::error::Error for ClientError {}
impl VideoPlayer {
    pub fn new() -> Result<Self, ClientError> {
        gst::init().map_err(|_| ClientError { message: "Failed to initialize GStreamer".to_string() })?;

        let pipeline = Pipeline::new();

        let uridecodebin = gst::ElementFactory::make("uridecodebin")
            .property("uri", &"udp://127.0.0.1:5000")
            .build()
            .map_err(|_| ClientError { message: "Failed to create uridecodebin".to_string() })?;

        let video_convert = gst::ElementFactory::make("videoconvert")
            .build()
            .map_err(|_| ClientError { message: "Failed to create videoconvert".to_string() })?;

        let video_sink = gst::ElementFactory::make("autovideosink")
            .build()
            .map_err(|_| ClientError { message: "Failed to create autovideosink".to_string() })?;

        pipeline.add_many(&[&uridecodebin, &video_convert, &video_sink])
            .map_err(|_| ClientError { message: "Failed to add elements to pipeline".to_string() })?;

        // Collega uridecodebin e video_convert
        uridecodebin.connect_pad_added({
            let video_convert = video_convert.clone(); // Clone video_convert to avoid moving
            move |_, src_pad| {
                let sink_pad = video_convert.static_pad("sink").unwrap();
                src_pad.link(&sink_pad).expect("Failed to link pads");
            }
        });

        video_convert.link(&video_sink).map_err(|_| ClientError { message: "Failed to link video_convert to video_sink".to_string() })?;

        pipeline.set_state(State::Playing).map_err(|_| ClientError { message: "Failed to set pipeline to Playing".to_string() })?;

        Ok(Self {
            pipeline: Some(pipeline),
        })
    }

    pub fn stop(&mut self) {
        if let Some(ref pipeline) = self.pipeline {
            pipeline.set_state(State::Null).unwrap();
        }
        self.pipeline = None;
    }
}
