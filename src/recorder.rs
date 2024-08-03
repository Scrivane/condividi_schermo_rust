use gst::prelude::*;
use gst::{Pipeline, State};
use std::fmt;

#[cfg(target_os = "linux")]
use tokio::runtime::Runtime;
#[cfg(target_os = "linux")]
use ashpd::{
    desktop::{
        screencast::{CursorMode, Screencast, SourceType},
        PersistMode,
    },
    WindowIdentifier,
};

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
use std::env;
pub struct ScreenRecorder {
    pipeline: Option<Pipeline>,
    recording: bool,
}

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

        #[cfg(target_os = "linux")]
        let pipeline = Self::create_pipeline_linux()?;

        #[cfg(target_os = "windows")]
        let pipeline = Self::create_pipeline_windows()?;

        #[cfg(target_os = "macos")]
        let pipeline = Self::create_pipeline_macos()?;

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

    #[cfg(target_os = "linux")]
    fn create_pipeline_linux() -> Result<Pipeline, ServerError> {
        let rt = Runtime::new().map_err(|e| ServerError {
            message: format!("Failed to create Tokio runtime: {}", e),
        })?;

        if   env::var("WAYLAND_DISPLAY").is_ok(){

        let valnod = match rt.block_on(run()) {
            Ok(value) => value,
            Err(e) => {
                return Err(ServerError {
                    message: format!("Failed to run async screencast session: {}", e),
                })
            }
        };



        let pipeline_description = format!(
            r#"
                pipewiresrc path={} !
                videoscale !
                video/x-raw,width=1280,height=720 !
                videorate !
                video/x-raw,framerate=5/1 !
                videoconvert !
                video/x-raw,format=BGR !
                avimux !
                filesink location=./finalmente.avi sync=true
            "#,
            valnod
        );

        let pipeline = gst::parse::launch(&pipeline_description)
            .expect("Failed to parse pipeline");
        let pipeline = pipeline
            .dynamic_cast::<gst::Pipeline>()
            .expect("Failed to cast pipeline");

        pipeline.set_state(gst::State::Playing).expect("Failed to start the pipeline");

        println!("su linux all 'id {}", valnod);

        Ok(pipeline)


    }else  {

        let videosrc = gst::ElementFactory::make("ximagesrc")   //non funziona con wayland ma solo xdg open 
        .property("use-damage", false)
        .build()
        .map_err(|_| ServerError {
            message: "Failed to create pipewiresrc".to_string(),
        })?;

        Self::create_common_pipeline(videosrc)
  
        
    }



        
    }

    #[cfg(target_os = "windows")]
    fn create_pipeline_windows() -> Result<Pipeline, ServerError> {
        let videosrc = gst::ElementFactory::make("d3d11screencapturesrc")
            .property("monitor-index", &0)
            .build()
            .map_err(|_| ServerError{ message: "Failed to create d3d11screencapturesrc".to_string()})?;

        Self::create_common_pipeline(videosrc)
    }

    #[cfg(target_os = "macos")]
    fn create_pipeline_macos() -> Result<Pipeline, ServerError> {
        let videosrc = gst::ElementFactory::make("avfvideosrc")
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create avfvideosrc".to_string(),
            })?;

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

        let video_convert = gst::ElementFactory::make("videoconvert")
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create videoconvert".to_string(),
            })?;

        let video_encoder = gst::ElementFactory::make("x264enc")
            .property_from_str("tune", "zerolatency")
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create x264enc".to_string(),
            })?;

        let flvmux = gst::ElementFactory::make("flvmux")
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create flvmux".to_string(),
            })?;

        let filesink = gst::ElementFactory::make("filesink")
            .property("location", &"video.flv")
            .build()
            .map_err(|_| ServerError {
                message: "Failed to create filesink".to_string(),
            })?;

        let pipeline = Pipeline::new();
        pipeline.add_many(&[&videosrc, &capsfilter, &video_convert, &video_encoder, &flvmux, &filesink])
            .map_err(|_| ServerError {
                message: "Failed to add elements to pipeline".to_string(),
            })?;

        gst::Element::link_many(&[&videosrc, &capsfilter, &video_convert, &video_encoder, &flvmux, &filesink])
            .map_err(|_| ServerError {
                message: "Failed to link elements".to_string(),
            })?;

        Ok(pipeline)
    }

    pub fn start(&mut self) -> Result<(), String> {
        let pipeline = self.pipeline.as_ref().unwrap();
        pipeline.set_state(State::Playing)
            .map_err(|_| "Failed to set pipeline to Playing".to_string())?;
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