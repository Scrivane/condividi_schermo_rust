use std::sync::{Arc, Mutex};
use gst::prelude::*;
use gst::{Pipeline, State};
use cfg_if::cfg_if;
use crate::streamer::error::ServerError;


//inclusioni necessarie solo per macos
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};


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
            top: 400,
            bottom: 400,
            right: 400,
            left: 400,
        };

        #[cfg(target_os = "windows")]
        let pipeline = Self::create_pipeline_windows(capture_region, monitor_id)?;

        #[cfg(target_os = "linux")]
        let pipeline = Self::create_pipeline_linux_wayland()?;


        #[cfg(target_os = "macos")]
        let pipeline = Self::create_pipeline_macos(capture_region)?;


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

    #[cfg(target_os = "windows")]
    fn create_pipeline_windows(capture_region: DimensionToCrop, monitor_id: usize) -> Result<Pipeline, ServerError> {

        let videosrc = gst::ElementFactory::make("d3d11screencapturesrc")
            .property("show-cursor", true)
            .property("monitor-index", &(monitor_id as i32))
            .property("show-border", true)
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create d3d11screencapturesrc".to_string(),
            })?;

        Self::create_common_pipeline(videosrc, capture_region)
    }

    #[cfg(target_os = "linux")]
    async fn run() -> ashpd::Result<u32> {
        let proxy = Screencast::new().await?;
        let mut valnode: u32 = 0;

        let session = proxy.create_session().await?;
        proxy
            .select_sources(
                &session,
                CursorMode::Metadata,
                SourceType::Monitor | SourceType::Window,
                true,
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

    #[cfg(target_os = "linux")]
    fn create_pipeline_linux_wayland() -> Result<Pipeline, ServerError> {
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

        Self::create_common_pipeline(videosrc, DimensionToCrop {
            top: 0,
            bottom: 100,
            right: 400,
            left: 30,
        })
    }

    #[cfg(target_os = "macos")]
    fn create_pipeline_macos(capture_region: DimensionToCrop) -> Result<Pipeline, ServerError> {
        let videosrc = gst::ElementFactory::make("avfvideosrc")
            .property("capture-screen", true)
            .property("device-index", &0)
            .build()
            .map_err(|_| ServerError { message: "Failed to create avfvideosrc".to_string()})?;

            
        // Successivamente, passa la sorgente video alla funzione comune per creare il resto della pipeline
        Self::create_common_pipeline(videosrc, capture_region)
    }

    fn create_common_pipeline(videosrc: gst::Element, crop: DimensionToCrop) -> Result<Pipeline, ServerError> {
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

        //
        //CREO IL CAPS FILTER IN DUE PASSAGGI PERCHE MI DAVA PROBLEMI
        //
        let caps = gst::Caps::builder("video/x-raw")
            .build();


        let capsfilter = gst::ElementFactory::make("capsfilter")
            .property("caps", &caps)  
            .build()
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

        let x264enc = gst::ElementFactory::make("x264enc")
        .build()
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

        //
        //IL VIDEOSRC ADESSO LO AGGIUNGO DIRETTAMENTE QUI
        //
        pipeline.add_many(&[
            &videosrc,
            &capsfilter,
            &videocrop,
            &queue1,
            &videoconvert,
            &queue2,
            &x264enc,
            &queue3,
            &rtph264pay,
            &queue4,
            &udpmulticastsink
        ]).map_err(|_| ServerError {
            message: "Failed to add elements to pipeline".to_string(),
        })?;

        //
        //SARA DA CAMBIARE PER LINUX PENSO VISTO CHE AGGIUNGO VIDEOSRC PRIMA
        //HO RIMOSSO L'ELSE POICHE LO AGGIUNGO DIRETTAMENTE DOPO
        //
        
        
             #[cfg(target_os = "linux")] {
                gst::Element::link_many(&[
                    &videosrc,
                    &videoscale,
                    &capsfilterdim,
                    &videoRate,
                    &capsfilter,
                ]).map_err(|_| ServerError {
                    message: "Failed to link elements".to_string(),
                })?;
            }
            
        


        gst::Element::link_many(&[
            &videosrc,
            &capsfilter,
            &videocrop,
            &videoconvert,
            &queue1,
            &queue2,
            &x264enc,
            &queue3,
            &rtph264pay,
            &queue4,
            &udpmulticastsink
        ]).map_err(|err| ServerError {
            message: format!("Failed to link elements: {:?}", err),  // Inserisce l'errore dettagliato nel messaggio
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


//
//CAMBIATO GESTIONE DELLO START
//
    pub fn start(&mut self) -> Result<(), String> {

        //macos richiede che ogni applicazione grafica venga runnata sul main thread, queste righe di codice 
        //permettono di forzare il programma a fare ciò
        #[cfg(target_os = "macos")]
        unsafe {
            let _: () = msg_send![class!(NSApplication), sharedApplication];
        }


        let pipeline = self.pipeline.as_ref().ok_or_else(|| "Pipeline is not initialized".to_string())?;
    
        // Verifica lo stato corrente della pipeline, se è già su Playing ritorno un errore
        let (_, current_state, _) = pipeline.state(Some(gst::ClockTime::from_mseconds(100)));

        // Verifica se la pipeline è già in esecuzione
        if current_state == gst::State::Playing {
            return Err("Pipeline is already playing".to_string());
        }
        
            // Imposta la pipeline su Playing
            pipeline.set_state(State::Playing).map_err(|err| format!("Failed to set pipeline to Playing: {:?}", err))?;
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
