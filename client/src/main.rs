mod screen_client;
mod client_connection;
mod client_icon;
use std::{env, path::Path};

use std::net::IpAddr;

use iced::{self, widget::{button, column, container, text_input, image::Handle, row}, Alignment, Element, Length, Size, Task};
use screen_client::Client;
use client_icon::Icon;

#[derive(Default)]
struct ClientGui{
    room_number: String,

}

#[derive(Debug, Clone)]
enum ClientMessage{
    InputChangedClient(String),
    StartStreamRequest,
}

impl ClientGui{

    fn update(&mut self, message: ClientMessage) ->Task<ClientMessage> {
        match message {
            ClientMessage::InputChangedClient(s) => {
                self.room_number = s;
            },
            ClientMessage::StartStreamRequest => {
                let ip:IpAddr=self.room_number.clone().trim().parse::<IpAddr>().unwrap();
                let client_handle = std::thread::spawn(move || {
                    Client::start_client(ip).unwrap() // in futuro maneggia errori
                });


            },
            _ => (),
        }
        Task::none()
    }

    fn view(&self) -> Element<ClientMessage> {
        let value_client = &self.room_number;
        let valid_ip = is_ipv4(&self.room_number);
        // Ottieni la directory corrente
        let current_dir = env::current_dir().expect("Failed to get current directory");
        // Costruisci il percorso relativo
        let relative_path_cross = Path::new("client/src/icon_images/cross.png");
        let relative_path_checked = Path::new("client/src/icon_images/checked.png");
        // Combina la directory corrente con il percorso relativo, queste sono le directory per le icone da visualizzare
        let cross_path = current_dir.join(relative_path_cross);
        let checked_path = current_dir.join(relative_path_checked);

        let text_input_client = text_input("Insert the ip where you want to connect", &value_client)
        .width(300)
        .on_input(ClientMessage::InputChangedClient)
        .padding(25);

        let start_client_button;
        let checked_box;

        match valid_ip {
            true => {
                checked_box = container(Icon::new(Handle::from_path(checked_path)));
                start_client_button = button("Start to watch the stream")
                .padding(25)
                .width(390)
                .on_press(ClientMessage::StartStreamRequest);
            },
            false => {
                checked_box = container(Icon::new(Handle::from_path(cross_path)));
                start_client_button = button("Insert a valid ip")
                .width(390)
                .padding(25);
            },
        }

        let ip_row = row![]
        .align_y(Alignment::Center)
        .spacing(10)
        .push(text_input_client)
        .push(checked_box);
    
        let client_view = column![]
        .align_x(Alignment::Center)
        .spacing(10)
        .push(ip_row)
        .push(start_client_button);

        let final_container = container(client_view)
       .center_x(Length::Fill)
       .center_y(Length::Fill);


        return container(final_container).into();
    }

    
}

fn is_ipv4(addr: &str) -> bool {
    match addr.parse::<IpAddr>() {
        Ok(_) => true,
        Err(_) => false, 
    }
}

fn main() {
    iced::application("Client_gui", ClientGui::update, ClientGui::view)
    .theme(|_| iced::Theme::TokyoNight)
    .run();
}
