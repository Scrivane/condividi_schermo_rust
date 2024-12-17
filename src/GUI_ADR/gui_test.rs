use selector_draw::MyCanvas;
use display::Display;
use clap::{error};
use iced::daemon::DefaultStyle;
use iced::theme::palette::Background;
use iced::widget::canvas::{Frame, Geometry, Program};
use iced::widget::tooltip::Position;
use iced::widget::{self, button, center, container, image, pick_list, tooltip, Canvas};
use std::{thread, time::Duration};
use iced::window::Id;
use iced::{
    touch::Event::FingerMoved,
    event::{self, Event, Status}, 
    mouse::{self, Event::{ButtonPressed, ButtonReleased, CursorMoved}},
     Element, Point, Rectangle, Renderer, Theme};
use iced::widget::{
     checkbox, column, horizontal_space, radio, row,
    scrollable, slider, text, text_input, toggler, vertical_space
};
use iced::widget::{Button, Column, Container, Slider,Text};
use iced::{border, window, Alignment, Center, Color, ContentFit, Fill, Font, Length, Pixels, Size, Subscription};
use iced::{Task};
use iced::Border;

#[cfg(target_os = "linux")]
use ashpd::{
    desktop::{
        screencast::{CursorMode, Screencast, SourceType},
        PersistMode,
    },
    WindowIdentifier,
};


use display_info::DisplayInfo;
#[cfg(target_os = "linux")]
use repng::meta::palette;
use screenshots::Screen;
#[cfg(target_os = "linux")]
use zbus::fdo::Error;

use cfg_if::cfg_if;
use std::net::IpAddr;
use get_if_addrs::get_if_addrs;

use crate::streamer::client::StreamerClient;
use crate::StreamerState;

use iced::application;

use super::{display, selector_draw};

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

struct ScreenSharer {
    position: Position,
    user_type:  UserType,
    input_value_streamer: String,
    input_value_client: String,
    ips:    String,
    streamer_client: Option<StreamerClient>,
    streamer_state: Option<StreamerState>,
    valnode:u32,
    mouse_point: Point,
    first_point: Option<Point>,
    second_point: Option<Point>,
    is_selecting_area: bool,
    window_id: window::Id,
    application_state: ApplicationState,
    available_display: Vec<Display>,
    selected_screen: Option<Display>,
}

impl Default for ScreenSharer {
    fn default() -> Self {

        let screen = Screen::all().unwrap();
        let displays: Vec<Display> = screen
        .iter()
        .map(|screen| Display{
            id: screen.display_info.id,
            width: screen.display_info.width,
            height: screen.display_info.height,
            frequency: screen.display_info.frequency
        }).collect();

        Self{
            position: Position::Top,
            user_type: UserType::None,
            input_value_streamer: "".to_string(),
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
        }
    }
}

#[derive(Default,Debug)]
enum UserType{  // sposta in main qunado è ora, di default è clioent
    client,
    streamer,
    #[default]
    None,
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
                    self.user_type = UserType::client;
               
                    let ip:IpAddr=self.input_value_client.clone().trim().parse::<IpAddr>().unwrap();
                    let client_handle = std::thread::spawn(move || {
                        crate::start_client(ip).unwrap() // in futuro maneggia errori
                    });

                    if let Ok(client) = client_handle.join() {
                        self.streamer_client = Some(client);
                    }

                    println!("{:?}",&self.user_type);
       
            }
            Message::StartRecording => {
                
          
                println!("Starting recording {:?}",&self.user_type);


                match self.streamer_client  {
                    None => println!("failed! No client was started before clicking on recording "),
                    Some(ref mut client) => {
                        client.start_recording();
                        //println!("{} / {} = {}", dividend, divisor, quotient)
                    },
                    
                }

   
        }
        Message::StopRec => {
            println!("Stop recording {:?}",&self.user_type);


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
                self.user_type = UserType::None;

                println!("ho finito lo stream {:?}",&self.user_type);
            }
            Message::StreamerPressed => {
                self.user_type = UserType::streamer;
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

                // Start the streamer in a separate thread and store the result in self.streamer_state.

              
                let valnode:usize=self.valnode.clone().try_into().expect("can't convert into usize");
                let streamer_state = std::thread::spawn(move || {
                    crate::start_streamer(valnode).unwrap()
                });
                if let Ok(streamer) = streamer_state.join() {
                    self.streamer_state = Some(streamer);
                    println!("Streamer started.");
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
                let arcStreamerState =state.streamer_arc.lock();
                let resImgStream=arcStreamerState.expect("errore frov").share_static_image_end("end_stream_ai.png".to_string());
                match resImgStream {
                    Ok(())=> { println!("Streaming end stream image");
                    if let Some(state) = self.streamer_state.take() {
                        std::thread::spawn(move || {

                            thread::sleep(Duration::from_millis(40000));
                            crate::stop_streamer(state).expect("Failed to stop streamer");
                        });
                        println!("Streamer stopped.");
                        self.user_type = UserType::None;
                    } else {
                        println!("No active streamer to stop.");
                    }
                    
                },
                    Err(ServerError)=> println!("Server error while streaming static image"),
                    
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
                return Task::batch(vec![
                    window::change_mode(self.window_id, window::Mode::Fullscreen),   // Metti a schermo intero
                    Task::perform(async { true }, |_| Message::SetSelectingArea),
                ]);
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
                    let arcStreamerState =state.streamer_arc.lock();
                    let streamres=arcStreamerState.expect("errore frov").share_static_image_end("blank.png".to_string());
            },
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
        else{
            iced::Subscription::none()
        }
    }

    fn view(&self) -> Element<Message> {
       
       match self.application_state {
            ApplicationState::Start => {
                let initial_text = text("Hello, select if you want to stream or to watch someone else");
                let streamer_button = button("Start a new Streaming Session")
                .padding(30)
                .width(200)
                .on_press(Message::ChangeApplicationState(ApplicationState::Streamer));
                let client_button = button("Start a new Client Session")
                .padding(50)
                .width(200)
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
                let value_client = &self.input_value_client;

                let text_input_client = text_input("es.. 198.154.1.12", &value_client)
                .on_input(Message::InputChangedClient)
                .padding(10)
                .size(30); 

                let client_section=column![]
                .push("Write the ip adress of the sharer").push(text_input_client)
                .push_maybe(self.can_continue_client().then(|| {
                    padded_button("Connect to a screen sharing session").on_press(Message::ClientPressed)
                    })).push_maybe( (!self.can_continue_client()).then(|| {
                        "Invalid ip, try to insert an other one "
                    }));
                

                let mut client_section_started = Self::container("Client")
                .push(
                    "Currently receiving screencast",
                ).push(padded_button("End client")
                .on_press(Message::StopClientPressed));
                    if self
                .streamer_client
                .as_ref()
                .map_or_else(|| false, |client| client.get_is_rec()) ==false
                {
                    client_section_started=client_section_started.push(
                    padded_button("start recording").on_press(Message::StartRecording),
                );
                } else {
                    client_section_started=client_section_started.push(
                    padded_button("stop recording").on_press(Message::StopRec),
                );
                }
                let comeback_button = button("Return to the main menu")
                .on_press(Message::ChangeApplicationState(ApplicationState::Start));

                let content = row![]
                .push(client_section)
                .push(comeback_button);

                return content.into();
            },
            ApplicationState::Streamer => {
            if !self.is_selecting_area {
                let screens_list = pick_list(self.available_display.clone(),
                        self.selected_screen,
                    Message::ChangeSelectedScreen)
                    .width(500)
                    .padding(30)
                    .placeholder("Choose the screen to stream");
        
                
                cfg_if! {
                    if #[cfg(target_os = "linux")] {
                    let start_button =padded_button("Start sharing screen").on_press(Message::RetIdPipewire);
                        } else {
                    let start_button =padded_button("Start sharing screen").on_press(Message::StreamerPressed);
                        }
                    }
              let stremer_section_started =  Self::container("Streamer")
              .push(
                  "Currently streaming, a client can watch this stream on one of the following adresses ( be sure to be able to connect to one of those ip )",
              ).push(Text::new(&self.ips)).push(padded_button("Blank the streamed screen")
              .on_press(Message::SetBlankScreen) )
              .push(padded_button("End Stream")
              .on_press(Message::StopStreamerPressed) 
            
            );
        
               let selecting_area_button = button("Select the area to stream")
               .on_press(Message::ToggleSelectingArea);

               let comeback_button = button("Return to the main menu")
               .on_press(Message::ChangeApplicationState(ApplicationState::Start));
        
                let content: Element<_> = column![stremer_section_started, screens_list, selecting_area_button, start_button, comeback_button]
                    //.max_width(540)
                    .spacing(20)
                    .padding(20)
                    .into();                
                center(content).into()
            }
            else {
        
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
        
                //.the(Background::new(Color::TRANSPARENT, Color::WHITE));
                        
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


    fn can_continue_streamer(&self) -> bool {  //trafrorma in enum cosi da gestire monitor non inserito , monitor sbagliiato o buono


        let display_infos = DisplayInfo::all().unwrap();
        let mut monitor_ids = Vec::new();
        let mut id = 0;
    
        for display_info in display_infos {
            println!("Name : {}, number {}", display_info.name, id);
            monitor_ids.push(id);
            id += 1;
        }
    
        let mut selected_monitor = 0;
        


            match self.input_value_streamer.clone().trim().parse() {
                Ok(num) if num < monitor_ids.len() => {
                    selected_monitor = num;
                    return true;
                }
                _ => {
                    println!("Invalid input. Please enter a valid monitor number.");
                    return false;
                }
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
            true,  //was true 
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


fn padded_button<Message: Clone>(label: &str) -> Button<'_, Message> {
    button(text(label)).padding([12, 24])
}
