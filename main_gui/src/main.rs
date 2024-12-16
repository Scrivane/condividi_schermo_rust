use iced::{
     Alignment, Element, Length, Settings, Size, Subscription, Task, Theme 
};
use iced::widget::{button, column, container, text, Button, Column, Text};

use std::process::{Command as ShellCommand, Stdio};

pub fn main() -> iced::Result {
    iced::application("Main_GUI", MainGui::update, MainGui::view)
    .theme(|_| Theme::TokyoNight)
    .window_size(Size::new(400.0, 500.0))
    .run()
}

#[derive(Debug, Clone)]
enum Message {
    LaunchStreamer,
    LaunchClient,
}
#[derive(Default)]
struct MainGui;

impl MainGui {

    fn update(&mut self, message: Message) ->Task<Message> {
        match message {
            Message::LaunchStreamer => {
                let _ = ShellCommand::new("cargo")
                    .args(&["run", "-p", "streamer"])
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn()
                    .expect("Failed to launch streamer");
            }
            Message::LaunchClient => {
                let _ = ShellCommand::new("cargo")
                    .args(&["run", "-p", "client"])
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn()
                    .expect("Failed to launch client");
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        
        let main_text = iced::widget::text("Hello! Choose if you want to stream your screen or watch someone else!");

        let streamer_button = button("Start a Streaming session")
        .padding(20)
        .on_press(Message::LaunchStreamer);

        let client_button = button("Join a Streaming Session")
        .padding(20)
        .on_press(Message::LaunchClient);

       
        
        let view_column = column![]
        .align_x(Alignment::Center)
        .spacing(15)
        .push(main_text)
        .push(streamer_button)
        .push(client_button);

        let cont = container(view_column)
        .center_x(Length::Fill)
        .center_y(Length::Fill);

        return cont.into();
    }
}
