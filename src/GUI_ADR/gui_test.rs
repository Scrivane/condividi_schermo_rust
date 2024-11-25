use clap::error;
use iced::widget::tooltip::Position;
use iced::widget::{button, center, container, tooltip};
use iced::Element;
use iced::widget::{
     checkbox, column, horizontal_space, radio, row,
    scrollable, slider, text, text_input, toggler, vertical_space
};
use iced::widget::{Button, Column, Container, Slider,Text};
use iced::{Center, Color, Fill, Font, Pixels,Subscription};
use iced::{Task};
use ashpd::{
    desktop::{
        screencast::{CursorMode, Screencast, SourceType},
        PersistMode,
    },
    WindowIdentifier,
};


use display_info::DisplayInfo;
use zbus::fdo::Error;

use cfg_if::cfg_if;
use std::net::IpAddr;
use get_if_addrs::get_if_addrs;

use crate::streamer::client::StreamerClient;
use crate::StreamerState;



pub fn run_iced() -> iced::Result {
    iced::run("Tooltip - Iced", Tooltip::update, Tooltip::view)
}

#[derive(Default)]
struct Tooltip {
    position: Position,
    user_type:  UserType,
    input_value_streamer: String,
    input_value_client: String,
    ips:    String,
    streamer_client: Option<StreamerClient>,
    streamer_state: Option<StreamerState>,
    valnode:u32,
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
    RetIdPipewire,
    GotValNode(Result<u32,u32>),
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
                if let Some(mut player) = self.streamer_client.take() {
                    std::thread::spawn(move || {
                        crate::stop_client(player).unwrap(); // Handle error appropriately
                    });
                }
                self.user_type = UserType::None;

                println!("ho finot lo stream {:?}",&self.user_type);
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
            }
        }
        Task::none()
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
        let value_client = &self.input_value_client;
        let value_streamer = &self.input_value_streamer;

        let mut text_input_streamer = text_input("es.. 0", &value_streamer)
        .on_input(Message::InputChangedStreamer)
        .padding(10)
        .size(30);
        let mut text_input_client = text_input("es.. 198.154.1.12", &value_client)
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




        let content: Element<_> = column![ controls,]
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

fn position_to_text<'a>(position: Position) -> &'a str {
    match position {
        Position::FollowCursor => "Follow Cursor",
        Position::Top => "Top",
        Position::Bottom => "Bottom",
        Position::Left => "Left",
        Position::Right => "Right",
    }
}
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