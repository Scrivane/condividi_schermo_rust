use std::sync::{Arc, Mutex};
use gst::{prelude::*};
use gst::{Pipeline, State};
use cfg_if::cfg_if;
use crate::streamer::error::ServerError;




pub struct DimensionToCrop {
    pub top: i32,
    pub bottom: i32,
    pub right: i32,
    pub left: i32,
}

pub struct ScreenStreamer {
    pipeline: Option<Pipeline>,
    clients: Arc<Mutex<Vec<String>>>,
    is_streaming: bool,
    is_paused: bool,
    capture_region:DimensionToCrop,
    monitor_index: usize,
}

impl ScreenStreamer {
    
    pub fn new(dimension: DimensionToCrop, monitor_index: usize) -> Result<Self, ServerError> {  //for linux monitor id =valnode =extrainfo
        gst::init().map_err(|e| ServerError {
            message: format!("Failed to initialize GStreamer: {}", e),
        })?;

        let pipeline = Self::create_pipeline2(&dimension, monitor_index).expect("errore creazioen pipeline screenstremer");

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
            capture_region : dimension,
            monitor_index: monitor_index,
        })
    }


    fn create_pipeline2(crop: &DimensionToCrop, monitor_index: usize) -> Result<Pipeline, ServerError> {

        //Creazione dei videosource specializzate per ogni OS
        #[cfg(target_os = "windows")]
        let videosrc = gst::ElementFactory::make("d3d11screencapturesrc")
            .property("show-cursor", true)
            .property("monitor-index", monitor_index as i32)
            //.property("show-border", true)
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create d3d11screencapturesrc".to_string(),
            })?;

        #[cfg(target_os = "macos")]
        let videosrc = gst::ElementFactory::make("avfvideosrc")
            .property("capture-screen", true)
            .property("device-index", monitor_index as i32)
            .build()
            .map_err(|_| ServerError { message: "Failed to create avfvideosrc".to_string()})?;

        cfg_if! {
            if #[cfg(target_os = "linux")] {
             
                let videosrc = gst::ElementFactory::make("pipewiresrc")
                    .property("path", monitor_index.to_string())
                    .property("do-timestamp", true)
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


                let videoRate = gst::ElementFactory::make("videorate").property("max-rate", 30).property("drop-only", true)
                .build()
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

        let x264enc = gst::ElementFactory::make("x264enc")
            .property("bitrate", 5000  as u32) // Bitrate in kbps
            .property_from_str("speed-preset", "ultrafast") // Faster encoding
            .property_from_str("tune", "zerolatency") //For live streaming with low latency
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
        pipeline.add(&videosrc).map_err(|_| ServerError {
            message: "Failed to add videosrc to pipeline".to_string(),
        })?;

        cfg_if! {
            if #[cfg(target_os = "linux")] {
                pipeline.add_many(&[
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

    /* FUNZIONI ADD AND REMOVE CLIENT (NON SERVONO PIU)

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
    */

    pub fn update_clients(&self, client_list_str: String) {

        let client_list = client_list_str.split(',').map(|s| s.to_string()).collect();
        {
            let mut clients = self.clients.lock().unwrap();
            if client_list_str.is_empty(){
                *clients = Vec::new();
            }
            else{
                *clients = client_list;
            }
            
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
    pub fn restart(&mut self) -> bool {
        if let Some(ref oldpipeline) = self.pipeline {
            oldpipeline.set_state(gst::State::Null).map_err(|e| ServerError {
                message: format!("Failed to set old pipeline state to null"),
            }).expect("errore stopping old pipeline");
        
        }
        let pipe=ScreenStreamer::create_pipeline2(&self.capture_region, self.monitor_index).expect("Error reCreating the pipeline");

        //self.is_streaming = true;
        self.pipeline = Some(pipe);
        ScreenStreamer::update_multiudpsink(self);
        let restart_result = ScreenStreamer::start(self);

        match restart_result {
            Ok(_) => {
                println!("Streaming is restarting");
                return true;
            },
            Err(e) => {
                println!("Error in restarting the streaming: {}", e);
                return false;
            },
        }
    }

    pub fn stop(&mut self) {
        if let Some(ref pipeline) = self.pipeline {
            let _ = pipeline.set_state(State::Null).map(|_| ());
        }
        self.pipeline = None;
        self.is_streaming = false;
    }

    pub fn pause(&mut self) -> bool {
        if let Some(ref pipeline) = self.pipeline {
            if self.is_streaming {
                let pause_result = pipeline.set_state(State::Paused);
                match pause_result {
                    Ok(_) => {
                        println!("Pause the streaming correctly");
                        self.is_paused = true;
                        return true;
                    },
                    Err(e) => {
                        println!("Error in pausing the screen: {}", e);
                        return false;
                    },
                }
            }
        }
        println!("pipeline does not exist, cannot pause");
        return false;
    }


    pub fn un_pause(&mut self) -> bool {
        if let Some(ref pipeline) = self.pipeline {
            if self.is_streaming {
                let pause_result = pipeline.set_state(State::Playing);
                match pause_result {
                    Ok(_) => {
                        println!("UNPause the streaming correctly");
                        self.is_paused = true;
                        return true;
                    },
                    Err(e) => {
                        println!("Error in UNpausing the screen: {}", e);
                        return false;
                    },
                }
            }
        }
        println!("pipeline does not exist, cannot pause");
        return false;
    }





    pub fn share_static_image_end(&mut self,imagename: String) -> Result<(), ServerError> {
        if let Some(ref oldpipeline) = self.pipeline {
            oldpipeline.set_state(gst::State::Null).map_err(|e| ServerError {
                message: format!("Failed to set old pipeline state to null"),
            })?;
        
        }

        let image_path="src/images/".to_string()+imagename.trim();

       
        let clients = self.clients.lock().unwrap();

        if clients.len() == 0 { //se non ci sono clienti non serve fare nulla
            return Ok({});
        }
        
        
        let addresses: Vec<String> = clients.iter().map(|addr| addr.to_string()).collect();
        let addresses_str = addresses.join(",");

        let pipeline_description = format!(r#"
        multifilesrc location={} loop=true !
        pngdec !
        videorate !
        video/x-raw,framerate=60/1 !
        videoconvert !
        x264enc !
        rtph264pay !
        multiudpsink name=multiudpsink clients={}
    "#, &image_path,&addresses_str
);


let pipeline = gst::parse::launch(pipeline_description.as_str()).expect("Failed to parse pipeline to gst::parse");
// Start playing the pipeline
let new_pipeline = pipeline.dynamic_cast::<gst::Pipeline>().expect("Failed to cast pipeline to gst::Pipeline");


    new_pipeline.set_state(State::Playing).map_err(|_| "Failed to set pipeline of static image to Playing ".to_string());

        self.pipeline = Some(new_pipeline);
        self.is_streaming = true;


    
            Ok({})
        
    }
        
 
}
