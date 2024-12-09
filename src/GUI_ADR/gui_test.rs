use clap::{error};
use iced::daemon::DefaultStyle;
use iced::theme::palette::Background;
use iced::widget::canvas::{Frame, Geometry, Program};
use iced::widget::tooltip::Position;
use iced::widget::{self, button, center, container, tooltip, Canvas, image};

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
use iced::{border, window, Center, Color, ContentFit, Fill, Font, Length, Pixels, Subscription};
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

pub fn run_iced() -> iced::Result {
    iced::application("Ferris - Iced", Tooltip::update, Tooltip::view)
        .style(Tooltip::style).subscription(Tooltip::subscription).transparent(true)
        .theme(|_| Theme::TokyoNight)//.transparent(true)
        .run()
}

struct Tooltip {
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
}

impl Default for Tooltip {
    fn default() -> Self {
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
    ChangePosition,
    StreamerPressed,
    ClientPressed,
    StopStreamerPressed,
    StopClientPressed,
    InputChangedStreamer(String),
    InputChangedClient(String),
    #[cfg(target_os = "linux")]
    RetIdPipewire,
    GotValNode(Result<u32,u32>),
    PointUpdated(Point),
    FirstPoint,
    SecondPoint,
    ToggleSelectingArea,
    TakeScreenshot,
    SetSelectingArea,
}

impl Tooltip {


    fn update(&mut self, message: Message) ->Task<Message> {


        match message {
            Message::ChangePosition => {
                let position = match &self.position {
                    Position::Top => Position::Bottom,
                    Position::Bottom => Position::Left,
                    Position::Left => Position::Right,
                    Position::Right => Position::FollowCursor,
                    Position::FollowCursor => Position::Top,
                };

                self.position = position;
            }

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


            Message::StopClientPressed => {
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
                let id_screen: usize = self.input_value_streamer.clone().trim().parse().unwrap();
         
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

/* 
                fn start_streamer2(num_monitor: usize) -> Result<StreamerState, Box<dyn Error>> {
                    let (control_sender, control_receiver) = mpsc::channel();
                    let (client_sender, client_receiver) = mpsc::channel();
                
                    let streamer = ScreenStreamer::new(num_monitor).expect("errore creazione scren streamer");
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
                
                    Ok(StreamerState {
                        control_sender,
                        control_thread,
                        client_thread,
                        discovery_thread,
                        streamer_arc,
                    })
                } */


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
                
                if let Some(state) = self.streamer_state.take() {
                    std::thread::spawn(move || {
                        crate::stop_streamer(state).expect("Failed to stop streamer");
                    });
                    println!("Streamer stopped.");
                    self.user_type = UserType::None;
                } else {
                    println!("No active streamer to stop.");
                }
            }
            Message::InputChangedStreamer(input_value) => {
                self.input_value_streamer = input_value;
            }
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
                    Task::perform(async {}, |_| Message::TakeScreenshot), // Esegue un'operazione vuota (opzionale)
                    window::change_mode(self.window_id, window::Mode::Fullscreen),   // Metti a schermo intero
                    Task::perform(async { true }, |_| Message::SetSelectingArea),
                ]);


            },
            Message::TakeScreenshot => {
                /* let screens = Screen::all().unwrap();
    
                for screen in screens {
                    println!("capturer {screen:?}");
                    let image = screen.capture().unwrap();
                    image
                        .save("target/screen_preview.png")
                        .unwrap();

                } */
            }
            Message::SetSelectingArea => {
                self.first_point = None;
                self.second_point = None;
                self.is_selecting_area = true;
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
        else{
            iced::Subscription::none()
        }
    }

    fn view(&self) -> Element<Message> {
        /*let tooltip = tooltip(
            button("Start Streamer")
                .on_press(Message::ChangePosition),
            position_to_text(self.position),
            self.position,
        )
        .gap(10)
        .style(container::rounded_box);*/

    if !self.is_selecting_area {
        let value_client = &self.input_value_client;
        let value_streamer = &self.input_value_streamer;

        let text_input_streamer = text_input("es.. 0", &value_streamer)
        .on_input(Message::InputChangedStreamer)
        .padding(10)
        .size(30);
        let text_input_client = text_input("es.. 198.154.1.12", &value_client)
        .on_input(Message::InputChangedClient)
        .padding(10)
        .size(30); 
    cfg_if! {
        if #[cfg(target_os = "linux")] {
        let start_button =padded_button("Start sharing screen").on_press(Message::RetIdPipewire);
            } else {
        let start_button =padded_button("Start sharing screen").on_press(Message::StreamerPressed);
            }
        }
        


      let streamer_section=column![]
      
     // .push(padded_button("Start sharing screen").on_press(Message::StreamerPressed))
      .push("Write the id of the screen you want to stream from")
      .push(text_input_streamer)

      .push_maybe(self.can_continue_streamer().then(|| {
        start_button

    
        //on_press(Message::RetIdPipewire)
        
       // on_press(Message::StreamerPressed)
    })).push_maybe( (!self.can_continue_streamer()).then(|| {
        "Invalid screen id, try to insert again "
    }));    //rendi più carino
      
      

      let client_section=column![]
      .push("Write the ip adress of the sharer").push(text_input_client)



      .push_maybe(self.can_continue_client().then(|| {
        padded_button("Connect to a screen sharing session").on_press(Message::ClientPressed)
    })).push_maybe( (!self.can_continue_client()).then(|| {
        "Invalid ip, try to insert an other one "
    }));



      let stremer_section_started =  Self::container("Streamer")
      .push(
          "Currently streaming, a client can watch this stream on one of the following adresses ( be sure to be able to connect to one of those ip )",
      ).push(Text::new(&self.ips)).push(padded_button("End Stream")
      .on_press(Message::StopStreamerPressed) 
    
    );

  let client_section_started = Self::container("Client")
  .push(
      "Currently receiving screencast",
  ).push(padded_button("End client")
  .on_press(Message::StopClientPressed)   
);
  //.push(padded_button("Connect to a screen sharing session").on_press(Message::ClientPressed));;

  


   
       let controls:iced::widget::Row<'_, Message>=match self.user_type {
        UserType::None=>  {row![]
        .push(streamer_section)
        .push(horizontal_space())  //togli 2
        .push(client_section)}
      UserType::client=>  {row![]
        .push(client_section_started)}
        //.push(horizontal_space())
       // .push( clientSection) }
      UserType::streamer=> {row![]
        .push(stremer_section_started)}

       };

       let selecting_area_button = button("Select the area to stream")
       .on_press(Message::ToggleSelectingArea);



        let content: Element<_> = column![ controls, selecting_area_button]
            //.max_width(540)
            .spacing(20)
            .padding(20)
            .into();

        /*let scrollable = scrollable(
                container(
                    content
                )
                .center_x(Fill),
            );

        */


        
        center(content).into()
    }
    else {

        #[cfg(target_os = "macos")]
        let column = column![
            
                image("target/screen_preview.png")
        .width(Length::Fill)
        .height(Length::Fill)
        .content_fit(ContentFit::Cover)
         ];
        #[cfg(not(target_os = "macos"))]
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



struct MyCanvas{
    first_point: Option<Point>,
    second_point: Point,
}

impl<Message> Program<Message> for MyCanvas {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme, 
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        match self.first_point {
            Some(point) => {
                let canvas_size = iced::Size::new(
                    self.second_point.x - self.first_point.unwrap().x,
                     self.second_point.y - self.first_point.unwrap().y);
                frame.fill_rectangle(point,
                     canvas_size, 
                     Color::from_rgba(0.0, 0.2, 0.4, 0.5));
            },
            None => (),
        }
       

        vec![frame.into_geometry()]
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
