use iced::widget::tooltip::Position;
use iced::widget::{button, center, container, tooltip};
use iced::Element;
use iced::widget::{
     checkbox, column, horizontal_space, radio, row,
    scrollable, slider, text, text_input, toggler, vertical_space
};
use iced::widget::{Button, Column, Container, Slider,Text};
use iced::{Center, Color, Fill, Font, Pixels};


use display_info::DisplayInfo;


use std::net::IpAddr;



pub fn run_iced() -> iced::Result {
    iced::run("Tooltip - Iced", Tooltip::update, Tooltip::view)
}

#[derive(Default)]
struct Tooltip {
    position: Position,
    userType:  UserType,
    input_value_streamer: String,
    input_value_client: String,

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
    InputChangedStreamer(String),
    InputChangedClient(String),
}

impl Tooltip {
    fn update(&mut self, message: Message) {
        /*match message {
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
        }
        */

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
                
                    self.userType = UserType::client;
               
                    let ip:IpAddr=self.input_value_client.clone().trim().parse::<IpAddr>().unwrap();
                    std::thread::spawn(move || {

                    crate::start_client(ip);// non elegante
                    });

                    println!("{:?}",&self.userType);
       
            }
            Message::StreamerPressed => {
                
                self.userType = UserType::streamer;
                let idSreen:usize=self.input_value_streamer.clone().trim().parse().unwrap();
                std::thread::spawn(move || {
                    crate::start_streamer(idSreen);
                    
                });
                // non elegante
                //println!("{:?}",&self.userType);
   
        }

        Message::InputChangedStreamer(input_value) => {
            self.input_value_streamer = input_value;
        }
        Message::InputChangedClient(input_value) => {
            self.input_value_client = input_value;
        }
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
        let valueClient = &self.input_value_client;
        let valueStreamer = &self.input_value_streamer;

        let mut text_input_streamer = text_input("es.. 0", &valueStreamer)
        .on_input(Message::InputChangedStreamer)
        .padding(10)
        .size(30);
        let mut text_input_client = text_input("es.. 198.154.1.12", &valueClient)
        .on_input(Message::InputChangedClient)
        .padding(10)
        .size(30); 



      let stremerSection=column![]
      
     // .push(padded_button("Start sharing screen").on_press(Message::StreamerPressed))
      .push("Write the id of the screen you want to stream from")
      .push(text_input_streamer)

      .push_maybe(self.can_continue_streamer().then(|| {
        padded_button("Start sharing screen").on_press(Message::StreamerPressed)
    })).push_maybe( (!self.can_continue_streamer()).then(|| {
        "Invalid screen id, try to insert again "
    }));    //rendi più carino
      
      

      let clientSection=column![]
      .push("Write the ip adress of the sharer").push(text_input_client)



      .push_maybe(self.can_continue_client().then(|| {
        padded_button("Connect to a screen sharing session").on_press(Message::ClientPressed)
    })).push_maybe( (!self.can_continue_client()).then(|| {
        "Invalid ip, try to insert an other one "
    }));



      let stremer_section_started =  Self::container("Streamer")
      .push(
          "Currently streaming",
      ).push(padded_button("End Stream")
     // .on_press(Message::ClientPressed) fa nulla da implementare
    
    );

  let client_section_started = Self::container("Client")
  .push(
      "Currently receiving screencast",
  ).push(padded_button("End client")
  //.on_press(Message::ClientPressed)    fa nulla
);
  //.push(padded_button("Connect to a screen sharing session").on_press(Message::ClientPressed));;

  


   
       let controls:iced::widget::Row<'_, Message>=match self.userType {
        UserType::None=>  {row![]
        .push(stremerSection)
        .push(horizontal_space())
        .push( clientSection)}
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


fn padded_button<Message: Clone>(label: &str) -> Button<'_, Message> {
    button(text(label)).padding([12, 24])
}