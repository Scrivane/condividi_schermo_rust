use gst::prelude::*;
use gst::{Pipeline, State};
use std::env;
use std::fmt;
use std::{thread, time};
use gst::parse;

//prova
//mod wayland_screen_cast;
//

use tokio::runtime::Runtime; 
use ashpd::{
    desktop::{
        screencast::{CursorMode, Screencast, SourceType},
        PersistMode,
    },
    WindowIdentifier,
};


async fn run() -> ashpd::Result<u32> {
    let proxy = Screencast::new().await?;
    let mut valnode: u32=0;


    
    let session = proxy.create_session().await?;
    proxy
        .select_sources(
            &session,
            CursorMode::Metadata,
            SourceType::Monitor | SourceType::Window,
            true,
            None,
            PersistMode::DoNot,  //donot
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
        valnode=stream.pipe_wire_node_id();
    });
    Ok(valnode)
}

pub struct ScreenRecorder {
    pipeline: Option<Pipeline>,
    recording: bool,
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
impl ScreenRecorder {
    pub fn new() -> Result<Self, ServerError> {
        gst::init().unwrap();


        let pipeline = Pipeline::new();


        // Rileva il sistema operativo
        let os = env::consts::OS;


        let rt = Runtime::new().map_err(|e| ServerError { message: format!("Failed to create Tokio runtime: {}", e) })?;


        // Crea l'elemento della sorgente video
        let videosrc = match os {
            "windows" => {
                gst::ElementFactory::make("d3d11screencapturesrc")
                    .property("display-id", &0)
                    .build()
                    .map_err(|_| ServerError{ message: "Failed to create d3d11screencapturesrc".to_string()})?
            }
            "macos" => {
                gst::ElementFactory::make("avfvideosrc")
                    .build()
                    .map_err(|_| ServerError{ message: "Failed to create avfvideosrc".to_string()})?
            }
            "linux" => {


                let mut valnod = 0;

                let result = rt.block_on(async {
                    run().await
                });
                
                match result {
                    Ok(value) => {
                        valnod = value;  // Assuming `run` returns an integer value or a value that can be assigned to `valnod`
                    }
                    Err(e) => {
                        return Err(ServerError{ message: format!("Failed to run async screencast session: {}", e) });
                    }
                };




      
                
                    // installo xdg-desktop-portal-gnome , ci sono per gl altri sistemi operativi 
                 let src=gst::ElementFactory::make("pipewiresrc")
                .build()
                .map_err(|_| ServerError{ message: "Failed to create pipewiresrc".to_string()})?;
                let nodeidPos=valnod as i32; 
                src.set_property("fd", &nodeidPos);

             


                let pipeline_description = format!(r#"
                        pipewiresrc path={} !
                        videoscale !
                        video/x-raw,width=1280,height=720 !
                        videorate !
                        video/x-raw,framerate=5/1 !
                        videoconvert !
                        video/x-raw,format=BGR !
                        avimux !
                        filesink location=./finalmente.avi sync=true
                    "#, &nodeidPos
                );

    // Parse the pipeline description
    let pipeline = gst::parse::launch(pipeline_description.as_str()).expect("Failed to parse pipeline to gst::parse");

    // Start playing the pipeline
    let pipeline = pipeline.dynamic_cast::<gst::Pipeline>().expect("Failed to cast pipeline to gst::Pipeline");

    pipeline.set_state(gst::State::Playing).expect("Faild to start the pipeline");








                println!("su linusx all 'id {}",&nodeidPos); 
                //src.set_property("node.id", &valnod);
             


                 //   thread::sleep(time::Duration::from_millis(10));
                src
                
            //    src.set_property("fd", &capturable.fd.as_raw_fd());
     //   src.set_property("path", &format!("{}", capturable.path));


            /*     gst::ElementFactory::make("ximagesrc")   //non funziona con wayland ma solo xdg open 
                .property("use-damage", false)
                .build()
                    .map_err(|_| ServerError{ message: "Failed to create ximagesrc".to_string()})?
*/

                    
            }
            _ => {
                return Err(ServerError{ message: "OS non supportato".to_string()});
            }
        };

        let capsfilter = gst::ElementFactory::make("capsfilter")
            .property(
                "caps",
                gst::Caps::builder("video/x-raw")
                    .field("framerate", &gst::Fraction::new(30, 1))
                    .build(),
            )
            .build()
            .map_err(|_| ServerError { message: "Failed to create capsfilter".to_string() })?;


        let video_convert = gst::ElementFactory::make("videoconvert")
            .build()
            .map_err(|_| ServerError { message: "Failed to create videoconvert".to_string() })?;

        let video_encoder   = gst::ElementFactory::make("x264enc")
            .property_from_str("tune", "zerolatency")
            .build()
            .map_err(|_| ServerError { message: "Failed to create x264enc".to_string() })?;

        let flvmux = gst::ElementFactory::make("flvmux")
            .build()
            .map_err(|_| ServerError { message: "Failed to create flvmux".to_string() })?;


        let filesink = gst::ElementFactory::make("filesink")
            .property("location", &"video.flv")
            .build()
            .map_err(|_| ServerError { message: "Failed to create filesink".to_string() })?;


      
    pipeline.add_many(&[&videosrc, &capsfilter, &video_convert, &video_encoder, &flvmux, &filesink])
            .map_err(|_| ServerError {message: "Failed to add elements to pipeline".to_string()})?;
  

        //test con wayland er fuznionare su maggioranza linux
         /* 
        pipeline.add_many(&[&videosrc, &video_convert, &video_encoder, &flvmux, &filesink])
            .map_err(|_| ServerError {message: "Failed to add elements to pipeline".to_string()})?;


            gst::Element::link_many(&[&videosrc, &video_convert, &video_encoder, &flvmux, &filesink])
            .map_err(|_| ServerError {message: "Failed to link elements".to_string()})?;

            */

        // Collega gli elementi
         gst::Element::link_many(&[&videosrc, &capsfilter, &video_convert, &video_encoder, &flvmux, &filesink])
            .map_err(|_| ServerError {message: "Failed to link elements".to_string()})?;




        // Gestione degli eventi
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
            recording: false,
        })
    }



    pub fn start(&mut self) -> Result<(), String> {
        // Imposta la pipeline in stato di riproduzione
        let pipeline = self.pipeline.as_ref().unwrap();
        pipeline.set_state(State::Playing).map_err(|_| "Failed to set pipeline to Playing".to_string())?;
        self.recording = true;

        Ok(())
    }
    pub fn stop(&mut self) {
        if let Some(ref pipeline) = self.pipeline {
            pipeline.set_state(State::Null).map(|_| ());
        }
        self.pipeline = None;
        self.recording = false;
    }

}