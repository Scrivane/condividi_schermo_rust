use std::sync::{Arc, Mutex};
use gst::prelude::*;
use gst::{Pipeline, State};
use cfg_if::cfg_if;
use crate::streamer::error::ServerError;
use crate::connection::server;

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
use crate::connection::server::Server;

pub struct DimensionToCrop { //usa u32
    top: i32,
    bottom: i32,
    right: i32,
    left:  i32
}



pub struct ScreenStreamer {
    pipeline: Option<Pipeline>,
    clients: Arc<Mutex<Vec<String>>>,
    is_streaming: bool,
    is_paused: bool,

}

impl ScreenStreamer {
    pub fn new() -> Result<Self, ServerError> {
        gst::init().unwrap();




        //Qui avremo bisogno di un if che controlla se fare fullsize o crop
        let capture_region = DimensionToCrop{top:300,bottom:300,right:300,left:300};

        #[cfg(target_os = "windows")]
        let pipeline = Self::create_pipeline_windows(capture_region)?;


        #[cfg(target_os = "linux")]
        let pipeline=Self::create_pipeline_linux_wayland()?;


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
    fn create_pipeline_windows(capture_region: DimensionToCrop) -> Result<Pipeline, ServerError> {
        let videosrc = gst::ElementFactory::make("d3d11screencapturesrc")
            .property("show-cursor",true)
            .property("monitor-index", &0)
            .property("show-border", true)
            .build()
            .map_err(|_| ServerError { message: "Failed to create d3d11screencapturesrc".to_string()})?;


        Self::create_common_pipeline(videosrc, capture_region)



        //Self::create_common_pipeline(videosrc)
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
        let rt = Runtime::new().map_err(|e| ServerError {  //caromai rendi un thread a parte
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

        // thread::sleep(Duration::from_secs(3600));

        let videosrc = gst::ElementFactory::make("pipewiresrc")
            .property("path",valnod.to_string())
            // .property("monitor-index", &0)
            .build()
            .map_err(|_| ServerError { message: "Failed to create pipewiresrc".to_string()})?;

        Self::create_common_pipeline(videosrc, DimensionToCrop{top:0,bottom:100,right:400,left:30})

    }

    fn create_common_pipeline(videosrc: gst::Element,crop:DimensionToCrop) -> Result<Pipeline, ServerError> {


        let videocrop= gst::ElementFactory::make("videocrop")
            .property("bottom", &crop.bottom)
            .property("top", &crop.top)
            .property("left", &crop.left)
            .property("right", &crop.right)
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create rtph264pay".to_string(),
            })?;




        cfg_if! {
                if #[cfg(target_os = "linux")] {
                                    //linux

                                    let videoscale = gst::ElementFactory::make("videoscale").build()
                                    .map_err(|_| ServerError {
                                        message: "Failed to create videoscale".to_string(),
                                    })?;

                                    let capsfilterdim = gst::ElementFactory::make("capsfilter")
                                    .property(
                                        "caps",
                                        gst::Caps::builder("video/x-raw")
                                            .field("width", 1280).field("height",720 )
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


        //set properties to udp sink for connection
        let multiudpsink = gst::ElementFactory::make("multiudpsink")
            .property("host", &"224.1.1.1") //use a multicast address
            .property("port", &5000)
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create multiudpsink".to_string(),
            })?;
        */

        let udpmulticastsink = gst::ElementFactory::make("multiudpsink")
            .build()
            .map_err(|_| ServerError { message: "Failed to create multiudpsink".to_string() })?;


        let pipeline = Pipeline::new();
        pipeline.add(&videosrc).map_err(|_| ServerError {
            message: "Failed to add elements to pipeline for linux".to_string(),
        })?;

        cfg_if! {
            if #[cfg(target_os = "linux")] {
                 pipeline.add_many(&[

                    &videoscale,
                    &capsfilterdim,
                    &videoRate]).map_err(|_| ServerError {
                        message: "Failed to add elements to pipeline for linux".to_string(),
                    })?;
                 }
            }



        //add elements to the pipeline
        pipeline.add_many(&[

            &capsfilter,
            &videocrop, //casomai prova a spostare
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
            &capsfilter

        ]).map_err(|_| ServerError {
            message: "Failed to link elements".to_string(),
        })?;



                 }
            else{
                gst::Element::link(&videosrc,  &capsfilter).map_err(|_| ServerError {
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


    pub fn add_client(&self, ip: String) -> Result<(), ServerError> {
        let multiudpsink = self.pipeline.as_ref().unwrap().by_name("multiudpsink").unwrap();
        multiudpsink.emit_by_name::<()>("add", &[&ip, &5000]);
        self.clients.lock().unwrap().push(ip);
        Ok(())
    }

    pub fn remove_client(&self, ip: &str) -> Result<(), ServerError> {
        let multiudpsink = self.pipeline.as_ref().unwrap().by_name("multiudpsink").unwrap();
        multiudpsink.emit_by_name::<()>("remove", &[&ip, &5000]);
        self.clients.lock().unwrap().retain(|x| x != ip);
        Ok(())
    }



    pub fn start(&mut self) -> Result<(), String> {
        // imposta la pipeline in stato di riproduzione
        let pipeline = self.pipeline.as_ref().unwrap();
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

    pub fn pause(&mut self){
        //verifica che la pipeline sia esista e che lo streaming sia attivo
        //poi va in pausa
        if let Some(ref pipeline) = self.pipeline {
            if self.is_streaming {
                let _ = pipeline.set_state(State::Paused).map(|_| ());
                self.is_paused = true;
            }
        }

    }

}