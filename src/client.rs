use gst::prelude::*;
use gst::{ClockTime, Pipeline, State};
use std::{fmt, thread};
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct VideoPlayer {
    pipeline: Option<Pipeline>,
    is_streaming: Arc<Mutex<bool>>,
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





        // Imposta lo stato della pipeline in "pronta"
        pipeline.set_state(State::Ready).unwrap();


        let is_streming = Arc::new(Mutex::new(false));
        Ok(Self{
            pipeline: Some(pipeline),
            is_streaming: is_streming,
        })
    }

    pub fn start(&mut self) -> Result<(), ClientError> {
        if let Some(ref pipeline) = self.pipeline {
            pipeline.set_state(State::Playing)
                .map_err(|_| ClientError { message: "Failed to start playing".to_string() })?;

            let bus = pipeline.bus().unwrap();
            let is_streaming = Arc::clone(&self.is_streaming);
            let pipeline_clone = self.pipeline.clone();

            //NOT AN OPTIMAL SOLUTION
            //the thread check if there are new message in the bus, if there are not, probably the stream is ended
            //so close the render window
            thread::spawn(move || {
                let timeout = Duration::from_secs(5);
                let mut last_msg_time = std::time::Instant::now();


                loop {
                    match bus.timed_pop(ClockTime::from_seconds(timeout.as_secs())) {
                        Some(msg) => {
                            last_msg_time = std::time::Instant::now();
                            match msg.view() {
                                gst::MessageView::Eos(..) => {
                                    println!("End of stream");
                                    let mut streaming = is_streaming.lock().unwrap();
                                    *streaming = false;
                                    break;
                                }
                                gst::MessageView::Error(err) => {
                                    eprintln!(
                                        "Error received from element {:?}: {}",
                                        err.src().map(|s| s.path_string()),
                                        err.error()
                                    );
                                    eprintln!("Debugging information: {:?}", err.debug());
                                    let mut streaming = is_streaming.lock().unwrap();
                                    *streaming = false;
                                    break;
                                }
                                _ => (),
                            }
                        }
                        None => {
                            if last_msg_time.elapsed() >= timeout {
                                println!("No messages received for a while. Stream is ending.");
                                let mut streaming = is_streaming.lock().unwrap();
                                *streaming = false;
                                break;
                            }
                        }


                    }
                }
                if let Some(pipeline) = pipeline_clone {
                    pipeline.set_state(State::Null).unwrap();
                }
                println!("Closing render window.");
                ClientError { message: "Closing render window".to_string() };

            });

            let mut streaming = self.is_streaming.lock().unwrap();
            *streaming = true;
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
