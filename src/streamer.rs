use gst::prelude::*;
use gst::{Pipeline, State};
use std::fmt;

pub struct ScreenStreamer {
    pipeline: Option<Pipeline>,
    streaming: bool,
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
impl ScreenStreamer {
    pub fn new() -> Result<Self, ServerError> {
        gst::init().unwrap();


        #[cfg(target_os = "windows")]
        let pipeline = Self::create_pipeline_windows()?;


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
            streaming: false,
        })


    }

    #[cfg(target_os = "windows")]
    fn create_pipeline_windows() -> Result<Pipeline, ServerError> {
        let videosrc = gst::ElementFactory::make("d3d11screencapturesrc")
            .property("show-cursor",true)
            .property("monitor-index", &0)
            .build()
            .map_err(|_| ServerError { message: "Failed to create d3d11screencapturesrc".to_string()})?;

        Self::create_common_pipeline(videosrc)
    }

    fn create_common_pipeline(videosrc: gst::Element) -> Result<Pipeline, ServerError> {
        let capsfilter = gst::ElementFactory::make("capsfilter")
            .property(
                "caps",
                gst::Caps::builder("video/x-raw")
                    .field("framerate", &gst::Fraction::new(30, 1))
                    .build(),
            )
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create capsfilter".to_string(),
            })?;

        let queue1 = gst::ElementFactory::make("queue")
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create queue1".to_string(),
            })?;

        let videoconvert = gst::ElementFactory::make("videoconvert")
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create videoconvert".to_string(),
            })?;

        let queue2 = gst::ElementFactory::make("queue")
            .build()
            .map_err(
                |_| ServerError {
                    message: "Failed to create queue2".to_string(),
                },
            )?;


        let x264enc = gst::ElementFactory::make("x264enc")
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create x264enc".to_string(),
            })?;

        let queue3 = gst::ElementFactory::make("queue")
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create queue3".to_string(),
            })?;
        let rtph264pay = gst::ElementFactory::make("rtph264pay")
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create rtph264pay".to_string(),
            })?;
        let queue4 = gst::ElementFactory::make("queue")
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create queue4".to_string(),
            })?;

        /*
        //set properties to udp sink for connection
        let udpsink = gst::ElementFactory::make("udpsink")
            .property("host", &"127.0.0.1")
            .property("port", &5000)
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create udpsink".to_string(),
            })?;

         */


        //set properties to udp sink for connection
        let udpmulticastsink = gst::ElementFactory::make("udpsink")
            .property("host", &"224.1.1.1") //use a multicast address
            .property("port", &5000)
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create multiudpsink".to_string(),
            })?;


        let pipeline = Pipeline::new();

        //add elements to the pipeline
        pipeline.add_many(&[
            &videosrc,
            &capsfilter,
            &queue1,
            &videoconvert,
            &queue2,
            &x264enc,
            &queue3,
            &rtph264pay,
            &queue4,
            &udpmulticastsink,
        ]).map_err(|_| ServerError {
            message: "Failed to add elements to pipeline".to_string(),
        })?;


        gst::Element::link_many(&[
            &videosrc,
            &capsfilter,
            &queue1,
            &videoconvert,
            &queue2,
            &x264enc,
            &queue3,
            &rtph264pay,
            &queue4,
            &udpmulticastsink,
        ]).map_err(|_| ServerError {
            message: "Failed to link elements".to_string(),
        })?;



        Ok(pipeline)

    }



    pub fn start(&mut self) -> Result<(), String> {
        // Imposta la pipeline in stato di riproduzione
        let pipeline = self.pipeline.as_ref().unwrap();
        pipeline.set_state(State::Playing).map_err(|_| "Failed to set pipeline to Playing".to_string())?;
        self.streaming = true;

        Ok(())
    }
    pub fn stop(&mut self) {
        if let Some(ref pipeline) = self.pipeline {
            let _ = pipeline.set_state(State::Null).map(|_| ());
        }
        self.pipeline = None;
        self.streaming = false;
    }

}