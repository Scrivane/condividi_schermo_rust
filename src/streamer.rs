use gst::prelude::*;
use gst::{Pipeline, State};
use std::{fmt, thread};
use std::sync::{Arc, Mutex};
use cfg_if::cfg_if;
use gstreamer_rtsp_server::prelude::{RTSPMediaExt, RTSPMediaFactoryExt, RTSPMountPointsExt, RTSPServerExt, RTSPServerExtManual};
use gstreamer_rtsp_server::RTSPServer;
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
    is_streaming: bool,
    is_paused: bool,
    rtsp_server: Option<gstreamer_rtsp_server::RTSPServer>,
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

impl ScreenStreamer {
    pub fn new() -> Result<Self, ServerError> {
        gst::init().map_err(|e| ServerError {
            message: format!("Failed to initialize GStreamer: {}", e),
        })?;

        let capture_region = DimensionToCrop {
            top: 300,
            bottom: 300,
            right: 300,
            left: 300,
        };

        #[cfg(target_os = "windows")]
        let pipeline_string = Self::create_pipeline_windows_string(capture_region);

        #[cfg(target_os = "linux")]
        let pipeline_string = Self::create_pipeline_linux_string(capture_region)?;

        let rtsp_port = 8554;
        let mount_path = "/stream";

        let rtsp_server = gstreamer_rtsp_server::RTSPServer::new();
        rtsp_server.set_service(rtsp_port.to_string().as_str());

        let mount_points = rtsp_server.mount_points().ok_or_else(|| ServerError {
            message: "Failed to get mount points".to_string(),
        })?;

        let factory = gstreamer_rtsp_server::RTSPMediaFactory::new();
        factory.set_launch(&pipeline_string);
        mount_points.add_factory(mount_path, factory);


        rtsp_server.attach(None).map_err(|e| ServerError {
            message: format!("Failed to attach RTSP server: {}", e),
        })?;



        Ok(Self {
            pipeline:
            is_streaming: false,
            is_paused: false,
            rtsp_server: Some(rtsp_server),

        })
    }


    #[cfg(target_os = "windows")]
    fn create_pipeline_windows_string(capture_region: DimensionToCrop) -> String {
        format!(
            "d3d11screencapturesrc show-cursor=true monitor-index=0 show-border=true ! videocrop bottom={} top={} left={} right={}  ! videoconvert  ! x264enc  ! rtph264pay pt=96 name=pay0",
            capture_region.bottom, capture_region.top, capture_region.left, capture_region.right
        )
    }

    #[cfg(target_os = "windows")]
    fn create_pipeline_windows(capture_region: DimensionToCrop) -> String {
        let pipeline_string = Self::create_pipeline_windows_string(capture_region);
        pipeline_string
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
    fn create_pipeline_linux_string(capture_region: DimensionToCrop) -> String {
        format!(
            "pipewiresrc path={} ! videocrop bottom={} top={} left={} right={} ! queue ! videoconvert ! queue ! x264enc ! queue ! rtph264pay ! queue",
            capture_region.bottom, capture_region.top, capture_region.left, capture_region.right
        )
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

        let pipeline_string = Self::create_pipeline_linux_string(DimensionToCrop {
            top: 0,
            bottom: 100,
            right: 400,
            left: 30,
        });

        Self::create_common_pipeline(&pipeline_string)
    }



    pub fn start(&mut self) -> Result<(), String> {
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

    pub fn pause(&mut self) {
        if let Some(ref pipeline) = self.pipeline {
            if self.is_streaming {
                let _ = pipeline.set_state(State::Paused).map(|_| ());
                self.is_paused = true;
            }
        }
    }
    pub fn get_url(&self) -> Option<String> {
        if let Some(ref server) = self.rtsp_server {
            let uri = format!("rtsp://localhost:{}/stream", server.service());
            return Some(uri);
        }
        None
    }
}