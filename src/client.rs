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

        gst::init().unwrap();


        let pipeline = Pipeline::new();


        let udpsrc = gst::ElementFactory::make("udpsrc")
            .property("port", &5000)
            .property("caps", &gst::Caps::new_empty_simple("application/x-rtp"))
            .build()
            .map_err(|_| ClientError { message: "Failed to create element 'udpsrc'".to_string()})?;

        let queue1 = gst::ElementFactory::make("queue")
            .build()
            .map_err(|_| ClientError { message: "Failed to create element 'queue1'".to_string()})?;

        let rtph264depay = gst::ElementFactory::make("rtph264depay")
            .build()
            .map_err(|_| ClientError { message: "Failed to create element 'rtph264depay'".to_string()})?;

        let queue2 = gst::ElementFactory::make("queue")
            .build()
            .map_err(|_| ClientError { message: "Failed to create element 'queue2'".to_string()})?;

        let ffdec_h264 = gst::ElementFactory::make("avdec_h264")
            .build()
            .map_err(|_| ClientError { message: "Failed to create element 'avdec_h264'".to_string()})?;


        let queue3 = gst::ElementFactory::make("queue")
            .build()
            .map_err(|_| ClientError { message: "Failed to create element 'queue3'".to_string()})?;

        let autovideosink = gst::ElementFactory::make("autovideosink")
            .build()
            .map_err(|_| ClientError { message: "Failed to create element 'autovideosink'".to_string()})?;




        // Aggiungi gli elementi alla pipeline
        pipeline.add_many(&[
            &udpsrc,
            &queue1,
            &rtph264depay,
            &queue2,
            &ffdec_h264,
            &queue3,
            &autovideosink,
        ]).map_err(|_| ClientError { message: "Failed to add elements to pipeline".to_string()})?;

        // Collega gli elementi nella pipeline usando link_many
        gst::Element::link_many(&[
            &udpsrc,
            &queue1,
            &rtph264depay,
            &queue2,
            &ffdec_h264,
            &queue3,
            &autovideosink,
        ]).map_err(|_| ClientError { message: "Failed to link elements".to_string()})?;



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
        pipeline.set_state(State::Null).unwrap();

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
