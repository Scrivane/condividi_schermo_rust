use gst::prelude::*;
use gst::{Pipeline, State};
use std::fmt;
use crate::streamer::ServerError;

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
        // Inizializza GStreamer
        gst::init()?;

        // Crea una nuova pipeline
        let pipeline = Pipeline::new();

        // Crea gli elementi GStreamer
        let udpsrc = gst::ElementFactory::make("udpsrc")?;
        let queue1 = gst::ElementFactory::make("queue")?;
        let rtph264depay = gst::ElementFactory::make("rtph264depay")?;
        let queue2 = gst::ElementFactory::make("queue")?;
        let ffdec_h264 = gst::ElementFactory::make("ffdec_h264")?;
        let queue3 = gst::ElementFactory::make("queue")?;
        let autovideosink = gst::ElementFactory::make("autovideosink")?;

        // Imposta le proprietà di udpsrc
        udpsrc.set_property("port", &9002)?;
        udpsrc.set_property("caps", &gst::Caps::new_simple("application/x-rtp", &[]))?;

        // Aggiungi gli elementi alla pipeline
        pipeline.add_many(&[
            &udpsrc,
            &queue1,
            &rtph264depay,
            &queue2,
            &ffdec_h264,
            &queue3,
            &autovideosink,
        ])?;

        // Collega gli elementi nella pipeline usando link_many
        gst::Element::link_many(&[
            &udpsrc,
            &queue1,
            &rtph264depay,
            &queue2,
            &ffdec_h264,
            &queue3,
            &autovideosink,
        ])?;

        // Avvia la pipeline
        pipeline.set_state(State::Playing)?;

        // Attendi fino a quando non viene ricevuto un messaggio di errore o fine del flusso
        let bus = pipeline.bus().unwrap();
        for msg in bus.iter_timed(gst::ClockTime::NONE) {
            match msg.view() {
                gst::MessageView::Eos(..) => {
                    println!("End of stream");
                    break;
                }
                gst::MessageView::Error(err) => {
                    eprintln!(
                        "Error received from element {:?}: {}",
                        err.src().map(|s| s.path_string()),
                        err.error()
                    );
                    eprintln!("Debugging information: {:?}", err.debug());
                    break;
                }
                _ => (),
            }
        }

        // Arresta la pipeline
        pipeline.set_state(State::Null)?;

        Ok(Self{
            pipeline: Some(pipeline),
        })
    }

    pub fn start(&mut self) -> Result<(), ClientError> {
        if let Some(ref pipeline) = self.pipeline {
            pipeline.set_state(State::Playing).map_err(|_| ClientError { message: "Failed to start playing".to_string()})?;
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(ref pipeline) = self.pipeline {
            pipeline.set_state(State::Null).unwrap();
        }
        self.pipeline = None;
    }
}
