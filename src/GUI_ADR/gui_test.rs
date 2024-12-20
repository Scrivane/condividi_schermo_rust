use iced::{keyboard::{Event::KeyPressed, Key}, widget::image::Handle};
use selector_draw::MyCanvas;
use display::Display;
use icon::Icon;
use cropper::dimension_to_crop;
use iced::widget::{self, button, center, container, pick_list, Canvas, MouseArea};
use std::{thread, time::Duration};
use iced::{
    touch::Event::FingerMoved,
    event::{self, Event, Status}, 
    mouse::{self, Event::{ButtonPressed, ButtonReleased, CursorMoved}},
     Element, Point, Theme};
use iced::widget::{
     column, row, text, text_input, Column
};
use iced::{border, window, Alignment, Color, Length, Subscription};
use iced::Task;


#[cfg(target_os = "linux")]
use ashpd::{
    desktop::{
        screencast::{CursorMode, Screencast, SourceType},
        PersistMode,
    },
    WindowIdentifier,
};


#[cfg(target_os = "linux")]
use repng::meta::palette;
use screenshots::Screen;
#[cfg(target_os = "linux")]
use zbus::fdo::Error;

use std::net::IpAddr;
use get_if_addrs::get_if_addrs;

use crate::streamer::client::StreamerClient;
use crate::StreamerState;

use iced::application;

use super::{cropper, display, icon, selector_draw};

pub fn run_iced() -> iced::Result {
    iced::application("Ferris - Iced", ScreenSharer::update, ScreenSharer::view)
        .style(ScreenSharer::style).subscription(ScreenSharer::subscription).transparent(true)
        .theme(|_| Theme::TokyoNight)//.transparent(true)
        .run()
}

#[derive(Debug, Clone)]
enum ApplicationState{
    Start,
    Streamer,
    Client,
}

#[derive(Debug, Clone)]
enum StreamingState{
    Starting,
    Play,
    Pause,
}

struct ScreenSharer {
    input_value_client: String,
    ips: String,
    streamer_client: Option<StreamerClient>,
    streamer_state: Option<StreamerState>,
    valnode:u32,
    mouse_point: Point,
    first_point: Option<Point>,
    second_point: Option<Point>,
    is_selecting_area: bool,
    is_blank:bool,
    window_id: window::Id,
    application_state: ApplicationState,
    available_display: Vec<Display>,
    selected_screen: Option<Display>,
    streaming_state: StreamingState
}

impl Default for ScreenSharer {
    fn default() -> Self {

        let screen = Screen::all().unwrap();
        let displays: Vec<Display> = screen
        .iter()
        .map(|screen| Display{
            //aggiungere conteggio da 1 in su;
            id: screen.display_info.id,
            width: screen.display_info.width,
            height: screen.display_info.height,
            frequency: screen.display_info.frequency
        }).collect();

        Self{
            input_value_client: "".to_string(),
            ips: "".to_string(),
            streamer_client: None,
            streamer_state: None,
            valnode: 0,
            mouse_point: Point::ORIGIN,
            first_point: None,
            second_point: None,
            is_selecting_area: false,
            window_id: window::Id::unique(),
            application_state: ApplicationState::Start,
            available_display: displays,
            selected_screen: None,
            is_blank:false,
            streaming_state: StreamingState::Starting,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    StreamerPressed,
    ClientPressed,
    StopStreamerPressed,
    StopClientPressed,
    InputChangedClient(String),
    GotValNode(Result<u32,u32>),
    PointUpdated(Point),
    FirstPoint,
    SecondPoint,
    ToggleSelectingArea,
    SetSelectingArea,
    StartRecording,
    StopRec,
    ChangeApplicationState(ApplicationState),
    ChangeSelectedScreen(Display),
    SetBlankScreen,
    UnSetBlankScreen,
    PauseStreaming,
    ResumeStreaming,
    #[cfg(target_os = "linux")]
    RetIdPipewire,
}

impl ScreenSharer {


    fn update(&mut self, message: Message) ->Task<Message> {
        match message {
            Message::ChangeSelectedScreen(display) => {
                self.selected_screen = Some(display);
            },
            Message::ClientPressed => {
                    let ip:IpAddr=self.input_value_client.clone().trim().parse::<IpAddr>().unwrap();
                    let client_handle = std::thread::spawn(move || {
                        crate::start_client(ip).unwrap() // in futuro maneggia errori
                    });

                    if let Ok(client) = client_handle.join() {
                        self.streamer_client = Some(client);
                    }
            }
            Message::StartRecording => {
                match self.streamer_client  {
                    None => println!("failed! No client was started before clicking on recording "),
                    Some(ref mut client) => {
                        client.start_recording();
                        //println!("{} / {} = {}", dividend, divisor, quotient)
                    },
                }
        }
        Message::StopRec => {
            match self.streamer_client  {
                None => println!("failed! No client was started before clicking on stop recording "),
                Some(ref mut client) => {
                    client.stop_recording();
              
                },
            }
    }
            Message::StopClientPressed => {
                if let Some(player) = self.streamer_client.take() {
                    std::thread::spawn(move || {
                        crate::stop_client(player).unwrap(); // Handle error appropriately
                    });
                }
                if let Some(player) = self.streamer_client.take() {
                    std::thread::spawn(move || {
                        crate::stop_client(player).unwrap(); // Handle error appropriately
                    });
                }
            }
            Message::StreamerPressed => {
                let crop = dimension_to_crop(self.first_point, self.second_point, self.selected_screen);
                
                #[cfg(not(target_os = "linux"))]
                let id_screen: usize = self.selected_screen.unwrap().id as usize;

                match get_if_addrs() {
                    Ok(interfaces) => {
                        let mut all_ips = String::new();
                        println!("Bound to the following network interfaces:");
                        for iface in interfaces {
                            println!("Interface: {}, IP: {:?}", iface.name, iface.ip());
                            if iface.name != "lo" && iface.ip().to_string() != "::1" {
                                all_ips.push_str(&iface.ip().to_string());
                                all_ips.push_str(" , ");
                            }
                        }
                        self.ips = all_ips;
                    }
                    Err(e) => {
                        eprintln!("Error retrieving network interfaces: {}", e);
                    }
                }
                #[cfg(target_os = "linux")]
                let id_screen:usize=self.valnode as usize;

                // Start the streamer in a separate thread and store the result in self.streamer_state.
                let streamer_state = std::thread::spawn(move || {
                    crate::start_streamer(crop, id_screen).unwrap()
                });
                if let Ok(streamer) = streamer_state.join() {
                    self.streamer_state = Some(streamer);
                    println!("Streamer started.");
                    self.streaming_state = StreamingState::Play;
                }
                else {
                    println!("Streamer DID NOT started.");
                    
                } 
            }
            #[cfg(target_os = "linux")]
            Message::RetIdPipewire => {
                return Task::perform(
                    pipewirerec(),
                    Message::GotValNode,
                );
            }
            Message::GotValNode(r)=>{
                self.valnode=r.expect("Error in deciding valnode");
                println!("valnode2   :{}",self.valnode);
                return Task::perform (async { },
                    |_| Message::StreamerPressed,
                );
            }
            Message::StopStreamerPressed => {
                let  state =self.streamer_state.as_ref().unwrap();
                let arc_streamer_state =state.streamer_arc.lock();
                let res_img_stream=arc_streamer_state.expect("errore frov").share_static_image_end("end_stream_ai.png".to_string());
                match res_img_stream {
                    Ok(())=> { println!("Streaming end stream image");
                    if let Some(state) = self.streamer_state.take() {
                        std::thread::spawn(move || {
                            thread::sleep(Duration::from_millis(40000));
                            crate::stop_streamer(state).expect("Failed to stop streamer");
                        });
                        println!("Streamer stopped.");
                        self.streaming_state = StreamingState::Pause;
                    } else {
                        println!("No active streamer to stop.");
                    }
                    
                },
                    Err(e)=> println!("Server error while streaming static image: {}", e),
                    
                }
            },
            Message::InputChangedClient(input_value) => {
                self.input_value_client = input_value;
            },
            Message::PointUpdated(p) => {
                self.mouse_point = p
            },
            Message::FirstPoint => {
                self.first_point = Some(self.mouse_point)
            },
            Message::SecondPoint => {
                self.second_point = Some(self.mouse_point);
                println!("New Points saved: {}, {}", self.first_point.unwrap(), self.second_point.unwrap());
                self.is_selecting_area = false;
            },
            Message::ToggleSelectingArea => {
                match self.first_point {
                    Some(_) => {
                        self.first_point = None;
                        self.second_point = None;
                    },
                    None => {
                        return Task::batch(vec![
                            window::change_mode(self.window_id, window::Mode::Fullscreen),   // Metti a schermo intero
                            Task::perform(async { true }, |_| Message::SetSelectingArea),
                        ]);
                    },
                }
                
            },
            Message::SetSelectingArea => {
                self.first_point = None;
                self.second_point = None;
                self.is_selecting_area = true;
            },
            Message::ChangeApplicationState(state) => {
                self.application_state = state;
            },
            Message::SetBlankScreen => {
                let  state =self.streamer_state.as_ref().unwrap();
                    let arc_streamer_state =state.streamer_arc.lock();
              
                    let streamres=arc_streamer_state.expect("errore getting  arc").share_static_image_end("blank.png".to_string());
                    match streamres {
                        Ok(())=> self.is_blank=true,
                        Err(err) => println!("{:?}",&err)
                    }
            },
            Message::UnSetBlankScreen => {
                let  state =self.streamer_state.as_ref().unwrap();
                    let arc_streamer_state =state.streamer_arc.lock();
                    let streamres=arc_streamer_state.expect("errore  getting  arc").reStart();
                    match streamres {
                        Ok(())=> self.is_blank=false,
                        Err(err) => println!("{:?}",&err)
                        
                    }
            },
            Message::PauseStreaming => {
                let  state =self.streamer_state.as_ref().unwrap();
                    let arc_streamer_state =state.streamer_arc.lock();
                    let streamer=arc_streamer_state.expect("errore  getting  arc").pause();
            },
            Message::ResumeStreaming => {
                let  state =self.streamer_state.as_ref().unwrap();
                    let arc_streamer_state =state.streamer_arc.lock();
                    let streamer=arc_streamer_state.expect("errore  getting  arc").reStart();
            }
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.is_selecting_area {
            event::listen_with(|event, status, _queue| match (event, status) {
                (Event::Mouse(CursorMoved { position }), Status::Ignored)
                | (Event::Touch(FingerMoved { position, .. }), Status::Ignored) => {
                    Some(Message::PointUpdated(position))
                }
                (Event::Mouse(ButtonPressed(_)), Status::Ignored) => {
                    Some(Message::FirstPoint)
                }
                (Event::Mouse(ButtonReleased(_)), Status::Ignored) => {
                    Some(Message::SecondPoint)
                }
                _ => None,
            })
        }
        else {
            match  self.streaming_state{
                StreamingState::Starting => {
                    match self.application_state {
                        ApplicationState::Start => {
                            Subscription::none()
                        },
                        ApplicationState::Streamer => {
                            event::listen_with(|event, status, _queue| match (event, status) {
                                (Event::Keyboard(KeyPressed { key, modifiers, .. }), Status::Ignored)
                                    if key ==  Key::Character("s".into()) && modifiers.control() =>
                                {
                                    println!("faccio partire lo streaming");
                                    Some(Message::StreamerPressed) 
                                },
                                _ => None,
                            })
                        },
                        ApplicationState::Client => {
                            Subscription::none()
                        },
                    }
                },
                StreamingState::Play => {
                    event::listen_with(|event, status, _queue| match (event, status) {
                        (Event::Keyboard(KeyPressed { key, modifiers, .. }), Status::Ignored)
                            if key ==  Key::Character("p".into()) && modifiers.control() =>
                        {
                            println!("metto in pausa lo streaming");
                            Some(Message::PauseStreaming) 
                        },
                        _ => None,
                    })
                },
                StreamingState::Pause => {
                    event::listen_with(|event, status, _queue| match (event, status) {
                        (Event::Keyboard(KeyPressed { key, modifiers, .. }), Status::Ignored)
                            if key ==  Key::Character("r".into()) && modifiers.control() =>
                        {
                            println!("faccio ripartire lo streamer");
                            Some(Message::ResumeStreaming) 
                        },
                        _ => None,
                    })
                },
            }   
        }
    }

    fn view(&self) -> Element<Message> {
       
       match self.application_state {
            ApplicationState::Start => {
                let initial_text = text("Hello, select if you want to stream or to watch someone else");
                let streamer_button = button("Start a new Streaming Session")
                .padding(40)
                .width(400)
                .on_press(Message::ChangeApplicationState(ApplicationState::Streamer));
                let client_button = button("Start a new Client Session")
                .padding(40)
                .width(400)
                .on_press(Message::ChangeApplicationState(ApplicationState::Client));

                let final_column = column![]
                .push(initial_text)
                .push(streamer_button)
                .push(client_button)
                .spacing(10)
                .align_x(Alignment::Center);

                let content = container(final_column)
                .center_x(Length::Fill)
                .center_y(Length::Fill);

                return content.into();
            },
            ApplicationState::Client => {
                let main_text = text("Client")
                .size(50);

                let back_icon = Icon::new(Handle::from_path("src/images/left.png"));
                let back_area = MouseArea::new(back_icon)
                .on_press(Message::ChangeApplicationState(ApplicationState::Start))
                .interaction(mouse::Interaction::Pointer);

                let text_input_client = text_input("es.. 198.154.1.12", 
                &self.input_value_client)
                .on_input(Message::InputChangedClient)
                .padding(10)
                .size(40)
                .width(400); 

                let start_client_button;
                let client_icon;
                match self.can_continue_client() {
                    true => {
                        start_client_button = button("Connect to a screen sharing session")
                        .width(500)
                        .padding(30)
                        .style(button::success)
                        .on_press(Message::ClientPressed);
                        client_icon = Icon::new(Handle::from_path("src/images/checked.png"));
                    },
                    false => {
                        start_client_button = button("Connect to a screen sharing session")
                        .width(500)
                        .padding(30)
                        .style(button::danger);
                        client_icon = Icon::new(Handle::from_path("src/images/cross.png"));
                    },
                }

                let mut client_section_started = Self::container("Client")
                .push(
                    "Currently receiving screencast",
                ).push(button("End client")
                .on_press(Message::StopClientPressed));
                    if self
                .streamer_client
                .as_ref()
                .map_or_else(|| false, |client| client.get_is_rec()) ==false
                {
                    client_section_started=client_section_started.push(
                    button("start recording").on_press(Message::StartRecording),
                );
                } else {
                    client_section_started=client_section_started.push(
                    button("stop recording").on_press(Message::StopRec),
                );
                }

                let first_row = row![]
                .align_y(Alignment::Start)
                .push(back_area)
                .push(main_text)
                .spacing(20);

                let second_row = row![]
                .spacing(10)
                .push(text_input_client)
                .push(client_icon);

                let content = column![]
                .spacing(15)
                .push(first_row)
                .push(second_row)
                .push(start_client_button);

                return center(content).into();
            },
            ApplicationState::Streamer => {
            if !self.is_selecting_area {
            
                let main_text = text("Streamer")
                .size(50);
        
                let content;
                //cambio il content in base al fatto che stiamo streammando o no
                match self.streaming_state {
                    StreamingState::Starting => {
                        let back_icon = Icon::new(Handle::from_path("src/images/left.png"));
                        let back_area = MouseArea::new(back_icon)
                        .on_press(Message::ChangeApplicationState(ApplicationState::Start))
                        .interaction(mouse::Interaction::Pointer);

                        let first_row = row![back_area, main_text]
                        .spacing(20)
                        .align_y(Alignment::Start);

                        let screens_list = pick_list(self.available_display.clone(),
                        self.selected_screen,
                        Message::ChangeSelectedScreen)
                        .width(400)
                        .padding(30)
                        .placeholder("Choose the screen to stream");

                        let selecting_area_button;
                        match self.first_point {
                            Some(_) => {
                                selecting_area_button = button("Reset the area to FullScreen")
                                .padding(30)
                                .width(400)
                                .on_press(Message::ToggleSelectingArea);
                            },
                            None => {
                                selecting_area_button = button("Select the area to stream")
                                .padding(30)
                                .width(400)
                                .on_press(Message::ToggleSelectingArea);
                            },
                        }
                        
                        
                        let start_button;
                        let button_text = text("Start Streaming");

                        #[cfg(target_os = "linux")]
                        match self.selected_screen {
                            Some(_) => {
                                start_button = button(button_text)
                                .padding(30)
                                .width(400)
                                .style(button::success)
                                .on_press(Message::RetIdPipewire);
                            },
                            None => {
                                start_button = button(button_text)
                                .padding(30)
                                .width(400)
                                .style(button::danger);
                            },
                        }

                        #[cfg(not(target_os = "linux"))]
                        match self.selected_screen {
                            Some(_) => {
                                start_button = button(button_text)
                                .padding(30)
                                .width(400)
                                .style(button::success)
                                .on_press(Message::StreamerPressed);
                            },
                            None => {
                                start_button = button(button_text)
                                .padding(30)
                                .width(400)
                                .style(button::danger);
                            },
                        }
                       
                        content = column![]
                        .push(first_row)
                        .push(screens_list)
                        .push(selecting_area_button)
                        .push(start_button)
                        .spacing(20);
                    },
                    StreamingState::Play => {
                        let play_text = text( "Currently streaming, a client can watch this stream on one of the following adresses ( be sure to be able to connect to one of those ip )");
                        let ip_text = text(&self.ips);
                        
                        let blankbutton=match self.is_blank {
                            false=>  button("Blank the streamed screen").on_press(Message::SetBlankScreen)
                                    .width(400)
                                    .padding(30),
                            true=> button("Unblank the streamed screen").on_press(Message::UnSetBlankScreen)
                                    .width(400)
                                    .padding(30)
                        };

                        let pause_stream_button = button("Pause Stream")
                        .width(400)
                        .padding(30)
                        .on_press(Message::PauseStreaming);

                        let end_stream_button = button("End Stream")
                        .width(400)
                        .padding(30)
                        .on_press(Message::StopStreamerPressed);

                        content = column![]
                        .align_x(Alignment::Center)
                        .spacing(20)
                        .push(main_text)
                        .push(play_text)
                        .push(ip_text)
                        .push(blankbutton)
                        .push(pause_stream_button)
                        .push(end_stream_button);
                    },
                    StreamingState::Pause => {
                        let pause_text = text("The streaming is currently in pause");
                        let end_stream_button = button("End Stream")
                        .width(400)
                        .padding(30)
                        .on_press(Message::StopStreamerPressed);

                        content = column![]
                        .push(main_text)
                        .push(pause_text)
                        .push(end_stream_button);
                    },
                }              
                center(content).into()
            }
            else {
                //scelta della parte di screen da streammare
                let column = column![];        
                let over_text = text("Choose the area to stream")
                .color(Color::from_rgb(3.0, 0.0, 0.0));  //mettere uno sfonte oltro al testo senno non è carino  
        
                let my_canvas =
                    Canvas::new(MyCanvas{first_point:self.first_point,
                        second_point:self.mouse_point})
                        .width(Length::Fill)
                        .height(Length::Fill);
        
                let my_stack = widget::Stack::new()
                .width(Length::Fill)
                .height(Length::Fill)
                .push(column)
                .push(over_text)
                .push(my_canvas);
                
                let my_container = container(my_stack)
                .width(Length::Fill)
                .height(Length::Fill).style(|theme| {
                    let palette = theme.extended_palette();
        
                    container::Style::default()
                        .border(border::color(palette.background.strong.color).width(4))
                })
                .padding(4);
                return my_container.into();
            }
           }
       }
}

fn style(&self, theme: &Theme) -> application::Appearance {
    use application::DefaultStyle;

        if self.is_selecting_area {
            application::Appearance {
                background_color: Color::TRANSPARENT,
                text_color: theme.palette().text,
            }
        }
        else {
            Theme::default_style(theme)
        } 
}
    fn can_continue_client(&self) -> bool {  //valuta se l'ip inserito è valido "migliorabile controllando se è un ip raggiungibile"
        self.input_value_client.clone().trim().parse::<IpAddr>().is_ok()

    }
    fn container(title: &str) -> Column<'_, Message> {
        column![text(title).size(50)].spacing(20)
    }
}

#[cfg(target_os = "linux")]
async fn pipewirerec() -> Result<u32,u32>{
    let proxy = Screencast::new().await.expect("couln not start screencast proxi");
    let mut valnode: u32 = 0;
   
    let session = proxy.create_session().await.expect("couln not start screencast session");
    proxy
        .select_sources(
            &session,
            CursorMode::Metadata,
            SourceType::Monitor | SourceType::Window,
            false,  //useful to avoid selecting more monitors
            None,
            PersistMode::DoNot,
        )
        .await.expect("couln not start select sources");

    let response = proxy
        .start(&session, &WindowIdentifier::default())
        .await.expect("couln not start response")
        .response().expect("couln not end response");
    response.streams().iter().for_each(|stream| {
        println!("node id: {}", stream.pipe_wire_node_id());
        println!("size: {:?}", stream.size());
        println!("position: {:?}", stream.position());
        valnode = stream.pipe_wire_node_id();
    });
    println!("valnode: {:?}", valnode);
    Ok(valnode)
}