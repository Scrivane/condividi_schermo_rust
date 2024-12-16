use crate::server_error::ServerError;
use crate::connection_server::DiscoveryServer;
use std::error::Error;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use gst::prelude::*;
use gst::{Pipeline, State};
use cfg_if::cfg_if;
use iced::futures;
use iced::Subscription;

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

enum ControlMessage {
    Pause,
    Resume,
    Stop,
}

pub struct StreamerState {
    control_sender: mpsc::Sender<ControlMessage>,
    control_thread: thread::JoinHandle<()>,
    client_thread: thread::JoinHandle<()>,
    discovery_thread: thread::JoinHandle<()>,
    streamer_arc: Arc<Mutex<ScreenStreamer>>,
}

pub struct DimensionToCrop {
    pub top: i32,
    pub bottom: i32,
    pub right: i32,
    pub left: i32,
}

#[derive(Clone)]
pub struct ScreenStreamer {
    pipeline: Option<Pipeline>,
    clients: Arc<Mutex<Vec<String>>>,
    is_streaming: bool,
    is_paused: bool,
}

impl ScreenStreamer {
    pub fn new(extrainfo: usize, dimension: DimensionToCrop) -> Result<Self, ServerError> {  //for linux monitor id =valnode =extrainfo
        gst::init().map_err(|e| ServerError {
            message: format!("Failed to initialize GStreamer: {}", e),
        })?;

        let capture_region = dimension;

        let pipeline = Self::create_pipeline2(capture_region, extrainfo).expect("errore creazioen pipeline screenstremer");


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


    fn create_pipeline2(crop: DimensionToCrop, extra: usize) -> Result<Pipeline, ServerError> {

        //Creazione dei videosource specializzate per ogni OS
        #[cfg(target_os = "windows")]
        let videosrc = gst::ElementFactory::make("d3d11screencapturesrc")
            .property("show-cursor", true)
            .property("monitor-index", &(extra as i32))
            //.property("show-border", true)
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create d3d11screencapturesrc".to_string(),
            })?;

        #[cfg(target_os = "macos")]
        let videosrc = gst::ElementFactory::make("avfvideosrc")
            .property("capture-screen", true)
           // .property("device-index", &extra)
            .build()
            .map_err(|_| ServerError { message: "Failed to create avfvideosrc".to_string()})?;

        cfg_if! {
            if #[cfg(target_os = "linux")] {
             
                let videosrc = gst::ElementFactory::make("pipewiresrc")
                    .property("path", extra.to_string())
                    .build()
                    .map_err(|_| ServerError {
                        message: "Failed to create pipewiresrc".to_string(),
                    })?;
        }
    }




        let videobox = gst::ElementFactory::make("videobox")
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
            &videobox,
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
            &videobox,
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
        
    fn create_pipeline(crop: DimensionToCrop, device_index: usize) -> Result<Pipeline, ServerError> {

        //Creazione dei videosource specializzate per ogni OS
        #[cfg(target_os = "windows")]
        let videosrc = gst::ElementFactory::make("d3d11screencapturesrc")
            .property("show-cursor", true)
            .property("monitor-index", &(device_index as i32))
            //.property("show-border", true)
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create d3d11screencapturesrc".to_string(),
            })?;

        #[cfg(target_os = "macos")]
        let videosrc = gst::ElementFactory::make("avfvideosrc")
            .property("capture-screen", true)
          //  .property("device-index", &device_index)
            .build()
            .map_err(|_| ServerError { message: "Failed to create avfvideosrc".to_string()})?;

        cfg_if! {
            if #[cfg(target_os = "linux")] {
                async fn pipewirerec() -> ashpd::Result<u32> {
                    let proxy = Screencast::new().await?;
                    let mut valnode: u32 = 0;
            
                    let session = proxy.create_session().await?;
                    proxy
                        .select_sources(
                            &session,
                            CursorMode::Metadata,
                            SourceType::Monitor | SourceType::Window,
                            true,  //was true 
                            None,
                            PersistMode::DoNot,
                        )
                        .await?;
            
                    let response = proxy
                        .start(&session, &WindowIdentifier::default())
                        .await?
                        .response()?;
                    response.streams().iter().for_each(|stream| {
                        println!("node id: {}", stream.pipe_wire_node_id());
                        println!("size: {:?}", stream.size());
                        println!("position: {:?}", stream.position());
                        valnode = stream.pipe_wire_node_id();
                    });
                    Ok(valnode)
                }




                 let rt = Runtime::new().map_err(|e| ServerError {
                    message: format!("Failed to create Tokio runtime: {}", e),
                })?;
 




                let valnod = match rt.block_on(pipewirerec()) {
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

    pub fn start_streamer(num_monitor: usize, dimension: DimensionToCrop) -> Result<Arc<Mutex<ScreenStreamer>>, Box<dyn Error>> {
        let (control_sender, control_receiver) = mpsc::channel();
        let (client_sender, client_receiver) = mpsc::channel();
    
        let streamer = ScreenStreamer::new(num_monitor, dimension).expect("errore creazione scren streamer");
        let streamer_arc = Arc::new(Mutex::new(streamer));
    
        let mut discovery_server = DiscoveryServer::new(client_sender);
        let discovery_thread = thread::spawn(move || {
            println!("Starting discovery server...");
            discovery_server.run_discovery_listener().expect("Failed to run discovery server");
        });
    
        let streamer_arc_clone = Arc::clone(&streamer_arc);
        let control_thread = thread::spawn(move || {
            while let Ok(message) = control_receiver.recv() {
                let mut streamer = streamer_arc_clone.lock().unwrap();
                match message {
                    ControlMessage::Pause => streamer.pause(),
                    ControlMessage::Resume => streamer.start().unwrap(),
                    ControlMessage::Stop => {
                        streamer.stop();
                        break;
                    }
                }
            }
        });
    
        let streamer_arc_clone = Arc::clone(&streamer_arc);
        let client_thread = thread::spawn(move || {
            while let Ok(client_list) = client_receiver.recv() {
                let client_list_clone = client_list.clone();
                let streamer = streamer_arc_clone.lock().unwrap();
                streamer.update_clients(client_list);
                println!("Client list updated: {}", client_list_clone);
            }
        });
    
        {
            let mut streamer = streamer_arc.lock().unwrap();
            streamer.start().expect("error in starting the streamer");
            println!(
                "Streamer started\n\
                Press CTRL+C to stop the server\n\
                Press CTRL+P to pause the stream\n\
                Press CTRL+R to resume the stream"
            );
        }
    
        Ok(streamer_arc)
    }
}
