use std::sync::{Arc, Mutex};
use gst::prelude::*;
use gst::{Pipeline, State};
use cfg_if::cfg_if;
use crate::streamer::error::ServerError;

#[cfg(target_os = "linux")]
use ashpd::{
    desktop::{
        screencast::{CursorMode, Screencast, SourceType},
        PersistMode,
    },
    WindowIdentifier,
};
#[cfg(target_os = "linux")]
use tokio::runtime::Runtime;

pub struct DimensionToCrop {
    top: i32,
    bottom: i32,
    right: i32,
    left: i32,
}

pub struct ScreenStreamer {
    pipeline: Option<Pipeline>,
    clients: Arc<Mutex<Vec<String>>>,
    is_streaming: bool,
    is_paused: bool,
}

impl ScreenStreamer {
    pub fn new(monitor_id: usize) -> Result<Self, ServerError> {
        gst::init().map_err(|e| ServerError {
            message: format!("Failed to initialize GStreamer: {}", e),
        })?;

        let capture_region = DimensionToCrop {
            top: 300,
            bottom: 300,
            right: 300,
            left: 300,
        };

        let pipeline = Self::create_pipeline(capture_region, monitor_id)?;


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
            clients: Arc::new(Mutex::new(vec![])),
            is_streaming: false,
            is_paused: false,
        })
    }


    
    fn create_pipeline(crop: DimensionToCrop, device_index: usize) -> Result<Pipeline, ServerError> {

        //Creazione dei videosource specializzate per ogni OS
        #[cfg(target_os = "windows")]
        let videosrc = gst::ElementFactory::make("d3d11screencapturesrc")
            .property("show-cursor", true)
            .property("monitor-index", &(device_index as i32))
            .property("show-border", true)
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create d3d11screencapturesrc".to_string(),
            })?;

        #[cfg(target_os = "macos")]
        let videosrc = gst::ElementFactory::make("avfvideosrc")
            .property("capture-screen", true)
            .property("device-index", &device_index)
            .build()
            .map_err(|_| ServerError { message: "Failed to create avfvideosrc".to_string()})?;

        cfg_if! {
            if #[cfg(target_os = "linux")] {
                let rt = Runtime::new().map_err(|e| ServerError {
                    message: format!("Failed to create Tokio runtime: {}", e),
                })?;

                let valnod = match rt.block_on(Self::run()) {
                    Ok(value) => value,
                    Err(e) => {
                        return Err(ServerError {
                            message: format!("Failed to run async screencast session: {}", e),
                        })
                    }
                };

                let videosrc = gst::ElementFactory::make("pipewiresrc")
                    .property("path", valnod.to_string())
                    .build()
                    .map_err(|_| ServerError {
                        message: "Failed to create pipewiresrc".to_string(),
                    })?;
        }
    }




        let videocrop = gst::ElementFactory::make("videocrop")
            .property("bottom", &crop.bottom)
            .property("top", &crop.top)
            .property("left", &crop.left)
            .property("right", &crop.right)
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create videocrop".to_string(),
            })?;

        cfg_if! {
            if #[cfg(target_os = "linux")] {
                let videoscale = gst::ElementFactory::make("videoscale").build()
                    .map_err(|_| ServerError {
                        message: "Failed to create videoscale".to_string(),
                    })?;

                let capsfilterdim = gst::ElementFactory::make("capsfilter")
                    .property(
                        "caps",
                        gst::Caps::builder("video/x-raw")
                            .field("width", 1280).field("height", 720)
                            .build(),
                    ).build().map_err(|_| ServerError {
                        message: "Failed to create capsfilterdim".to_string(),
                    })?;

                let videoRate = gst::ElementFactory::make("videorate").build()
                    .map_err(|_| ServerError {
                        message: "Failed to create videoRate".to_string(),
                    })?;
            }
        }

        let capsfilter = gst::ElementFactory::make("capsfilter")
            .property(
                "caps",
                gst::Caps::builder("video/x-raw")
                    .field("framerate", &gst::Fraction::new(30, 1))
                    .build(),
            ).build()
            .map_err(|_| ServerError {
                message: "Failed to create capsfilter".to_string(),
            })?;

        let queue1 = gst::ElementFactory::make("queue").build()
            .map_err(|_| ServerError {
                message: "Failed to create queue1".to_string(),
            })?;

        let videoconvert = gst::ElementFactory::make("videoconvert").build()
            .map_err(|_| ServerError {
                message: "Failed to create videoconvert".to_string(),
            })?;

        let queue2 = gst::ElementFactory::make("queue").build()
            .map_err(|_| ServerError {
                message: "Failed to create queue2".to_string(),
            })?;

        let x264enc = gst::ElementFactory::make("x264enc").build()
            .map_err(|_| ServerError {
                message: "Failed to create x264enc".to_string(),
            })?;

        let queue3 = gst::ElementFactory::make("queue").build()
            .map_err(|_| ServerError {
                message: "Failed to create queue3".to_string(),
            })?;

        let rtph264pay = gst::ElementFactory::make("rtph264pay").build()
            .map_err(|_| ServerError {
                message: "Failed to create rtph264pay".to_string(),
            })?;

        let queue4 = gst::ElementFactory::make("queue").build()
            .map_err(|_| ServerError {
                message: "Failed to create queue4".to_string(),
            })?;

        let udpmulticastsink = gst::ElementFactory::make("multiudpsink")
            .property("clients", "")
            .name("multiudpsink")
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create multiudpsink".to_string(),
            })?;

        let pipeline = Pipeline::new();
        pipeline.add(&videosrc).map_err(|_| ServerError {
            message: "Failed to add videosrc to pipeline".to_string(),
        })?;

        cfg_if! {
            if #[cfg(target_os = "linux")] {
                pipeline.add_many(&[
                    &videoscale,
                    &capsfilterdim,
                    &videoRate,
                ]).map_err(|_| ServerError {
                    message: "Failed to add elements to pipeline for linux".to_string(),
                })?;
            }
        }

        pipeline.add_many(&[
            &capsfilter,
            &videocrop,
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

        cfg_if! {
            if #[cfg(target_os = "linux")] {
                gst::Element::link_many(&[
                    &videosrc,
                    &videoscale,
                    &capsfilterdim,
                    &videoRate,
                    &capsfilter,
                ]).map_err(|_| ServerError {
                    message: "Failed to link elements".to_string(),
                })?;
            } else {
                gst::Element::link(&videosrc, &capsfilter).map_err(|_| ServerError {
                    message: "Failed to link elements".to_string(),
                })?;
            }
        }

        gst::Element::link_many(&[
            &capsfilter,
            &videocrop,
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



    pub fn add_client(&mut self, client_addr: String) {
        {
            let mut clients = self.clients.lock().unwrap();
            clients.push(client_addr.clone());
        }

        println!("Added client: {}", client_addr);

        // Aggiorna il multiudpsink
        self.update_multiudpsink();
    }

    pub fn remove_client(&self, client_addr: String) -> Result<(), ServerError> {
        {
            let mut clients = self.clients.lock().map_err(|e| ServerError {
                message: format!("Failed to lock clients mutex: {}", e),
            })?;
            if let Some(index) = clients.iter().position(|addr| addr == &client_addr) {
                clients.remove(index);
            }
        }

        self.update_multiudpsink();
        println!("Removed client: {}", client_addr);
        Ok(())
    }

    pub fn update_clients(&self, client_list_str: String) {
        let client_list = client_list_str.split(',').map(|s| s.to_string()).collect();
        {
            let mut clients = self.clients.lock().unwrap();
            *clients = client_list;
        }
        self.update_multiudpsink();
    }

    fn update_multiudpsink(&self) {
        if let Some(pipeline) = &self.pipeline {
            let multiudpsink = pipeline
                .by_name("multiudpsink")
                .expect("Multiudpsink element not found");


            let clients = self.clients.lock().unwrap();
            let addresses: Vec<String> = clients.iter().map(|addr| addr.to_string()).collect();
            let addresses_str = addresses.join(",");


            multiudpsink
                .set_property("clients", &addresses_str);


        }
    }


    pub fn start(&mut self) -> Result<(), String> {
        let pipeline = self.pipeline.as_ref().ok_or_else(|| "Pipeline is not initialized".to_string())?;
        pipeline.set_state(State::Playing).map_err(|_| "Failed to set pipeline to Playing".to_string())?;
        self.is_streaming = true;

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(ref pipeline) = self.pipeline {
            let _ = pipeline.set_state(State::Null).map(|_| ());
        }
        self.pipeline = None;
        self.is_streaming = false;
    }

    pub fn pause(&mut self) {
        if let Some(ref pipeline) = self.pipeline {
            if self.is_streaming {
                let _ = pipeline.set_state(State::Paused).map(|_| ());
                self.is_paused = true;
            }
        }
    }
}
