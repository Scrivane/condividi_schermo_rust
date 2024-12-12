mod display;
mod connection_server;
mod screen_streamer;
mod server_error;
mod selector;

use iced::{
    alignment::Vertical::Bottom, application, event::{self, Event, Status}, keyboard::{self, key::Named, Event::KeyPressed, Key, Modifiers}, mouse::Event::{ButtonPressed, ButtonReleased, CursorMoved}, touch::Event::FingerMoved, widget::{self, button, column, container, pick_list, row, text, text_input, Canvas, Container}, Alignment, Background, Color, Element, Length, Point, Subscription, Task, Theme};

#[cfg(target_os = "linux")]
use ashpd::{
    desktop::{
        screencast::{CursorMode, Screencast, SourceType},
        PersistMode,
         },WindowIdentifier};
#[cfg(target_os = "linux")]
 use zbus::fdo::Error;

use cfg_if::cfg_if;
use std::{net::IpAddr, sync::{Arc, Mutex}};
use get_if_addrs::get_if_addrs;

use screenshots::Screen;
use display::Display;
use screen_streamer::{DimensionToCrop, ScreenStreamer, StreamerState};
use selector::ScreenSelector;

enum StreamingState{
    Play,
    Pause,
}
struct Streamer{
    is_selecting_area: bool,
    mouse_point: Point,
    first_point: Option<Point>,
    second_point: Option<Point>,
    available_display: Vec<Display>,
    selected_screen: Option<Display>,
    room_number: String,
    ips: String,
    valnode: u32,
    streamer_state: Option<StreamingState>,
    streamer: Option<Arc<Mutex<ScreenStreamer>>>
}

#[derive(Debug, Clone)]
enum StreamerMessagge{
    PointUpdated(Point),
    FirstPoint,
    SecondPoint,
    ToggleSelectingArea,
    ChangeSelectedScreen(Display),
    RoomNumberChanged(String),
    StartStreaming,
    PauseStreaming,
    ResetSelectedArea,
}

impl Default for Streamer {
    fn default() -> Self {
       //prendo tutte le informazioni degli schermi disponibili
       let screen = Screen::all().unwrap();
       let displays: Vec<Display> = screen
       .iter()
       .map(|screen| Display{
        id: screen.display_info.id,
        width: screen.display_info.width,
        height: screen.display_info.height,
        frequency: screen.display_info.frequency
       }).collect();

        println!("Lista di monitor {:?}", displays);

        Self{
            is_selecting_area: false,
            mouse_point: Point::ORIGIN,
            first_point: None,
            second_point: None,
            available_display: displays,
            selected_screen: None,
            room_number: "".to_string(),
            ips: "".to_string(),
            valnode: 0,
            streamer_state: None,
            streamer: None
        }
    }
}

impl Streamer {

    fn update(&mut self, message: StreamerMessagge) ->Task<StreamerMessagge> {
        match message {
            StreamerMessagge::PointUpdated(point) => {
                self.mouse_point = point
            },
            StreamerMessagge::FirstPoint => {
                self.first_point = Some(self.mouse_point)
            },
            StreamerMessagge::SecondPoint => {
                self.second_point = Some(self.mouse_point);
                println!("New Points saved: {}, {}", self.first_point.unwrap(), self.second_point.unwrap());
                self.is_selecting_area = false;
            },
            StreamerMessagge::ChangeSelectedScreen(display) => {
                self.selected_screen = Some(display);
            },
            StreamerMessagge::RoomNumberChanged(s) => 
            {
                self.room_number = s;
            },
            StreamerMessagge::ToggleSelectingArea => {
                self.is_selecting_area = true;
            },
            StreamerMessagge::ResetSelectedArea => {
                self.first_point = None;
                self.second_point = None;
            },
            StreamerMessagge::StartStreaming => {
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
                let dimension_to_crop = self.dimension_to_crop();

                 // Start the streamer in a separate thread and store the result in self.streamer_state
                 let valnode:usize=self.selected_screen.unwrap().id.clone().try_into().expect("can't convert into usize");
                  // Crea un Arc<Mutex<Option<ScreenStreamer>>> per condividere in modo sicuro tra i thread
                let handle = std::thread::spawn(move || {
                  Some(screen_streamer::ScreenStreamer::start_streamer(valnode, dimension_to_crop).unwrap());
                });

                match handle.join() {
                    Ok(_) => {
                        println!("Streamer Started!")
                    },
                    Err(e) => {
                        println!("Streamer had a problem {:?}", e)
                    },
                }
                
               
            },
            _ => {},
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<StreamerMessagge> {
        if self.is_selecting_area {
            event::listen_with(|event, status, _queue| match (event, status) {
                (Event::Mouse(CursorMoved { position }), Status::Ignored)
                | (Event::Touch(FingerMoved { position, .. }), Status::Ignored) => {
                    Some(StreamerMessagge::PointUpdated(position))
                }
                (Event::Mouse(ButtonPressed(_)), Status::Ignored) => {
                    Some(StreamerMessagge::FirstPoint)
                }
                (Event::Mouse(ButtonReleased(_)), Status::Ignored) => {
                    Some(StreamerMessagge::SecondPoint)
                }
                _ => None,
            })
        } else {
            event::listen_with(|event, status, _queue| match (event, status) {
                (Event::Keyboard(KeyPressed { key, modifiers, .. }), Status::Ignored)
                    if key ==  Key::Character("p".into()) && modifiers.control() =>
                {
                    println!("metto in pausa lo streaming");
                    Some(StreamerMessagge::ToggleSelectingArea) // Assicurati che StreamerMessagge abbia la variante `Pause`
                },
                _ => None,
            })
        }
    }
    
    

    fn view(&self) -> Element<StreamerMessagge> {

        if self.is_selecting_area {
            let column = column![];
        


            let over_text = text("Choose the area to stream")
            .color(Color::from_rgb(3.0, 0.0, 0.0));  //mettere uno sfonte oltro al testo senno non è carino  
    
            let my_canvas =
                Canvas::new(ScreenSelector{
                    first_point:self.first_point,
                    second_point:self.mouse_point})
                    .width(Length::Fill)
                    .height(Length::Fill);
    
            let my_stack = widget::Stack::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .push(column)
            .push(over_text)
            .push(my_canvas);

        return my_stack.into();
            
        }

        match self.streamer_state {
            Some(StreamingState::Play) => {
                let mut ips_copy = self.ips.clone();
                let ip_text = text("You are now Streaming, to watch to this stream please connect yourself to the following ip: ");
                let s_text = text(ips_copy.split_off(1).to_string().clone());

                let final_container = container(row![ip_text, s_text]);
                return final_container.into();
            },
            Some(StreamingState::Pause)=> {
                return container("The streaming is in pause").into();
            },
            None => {
                 //creo la pick list per la selezione schermo
                        let screens_list = pick_list(self.available_display.clone(),
                        self.selected_screen,
                    StreamerMessagge::ChangeSelectedScreen)
                    .width(500)
                    .padding(30)
                    .placeholder("Choose the screen to stream");
                    
                    let choose_area_button;

                    match self.first_point {
                        None => {
                            choose_area_button = button("Choose the area to stream")
                            .padding(30)
                            .width(500)
                            .on_press(StreamerMessagge::ToggleSelectingArea);
                        }
                        Some(_point) => {
                            choose_area_button = button("Reset the area to stream at FullScreen")
                            .padding(30)
                            .width(500)
                            .on_press(StreamerMessagge::ResetSelectedArea);
                        }
                    }

                    let my_style = button::Style{
                        background: Some(Background::Color(Color::BLACK)),
                        ..Default::default()
                    };

                    let start_streaming_button;

                    if  self.selected_screen != None {
                        start_streaming_button = button("Start the streaming")
                        .width(500)
                        .padding(30)
                        .on_press(StreamerMessagge::StartStreaming)
                        .style(button::success);
                    }
                    else {
                        start_streaming_button = button("You cannot start the stream if you do not choose a room to stream and a display")
                        .width(500)
                        .padding(30)
                        .style(button::danger);
                    }

                    

                    let final_column = column![]
                    .spacing(15)
                    .align_x(Alignment::Center)
                    .push(screens_list)
                    .push(choose_area_button)
                    .push(start_streaming_button);

                    let final_container = container(final_column)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill);

                     return final_container.into();
            }
        }
       
    } 

    fn dimension_to_crop(&self) -> DimensionToCrop
{
    match self.first_point{
        Some(point) => {
            //calculate the Dimension to stream
            let top_crop = (point.y - Point::ORIGIN.y).abs();
            let bottom_crop = (self.selected_screen.unwrap().height as f32 - self.second_point.unwrap().y).abs();
            let right_crop = (self.selected_screen.unwrap().width as f32 - self.second_point.unwrap().x).abs();
            let left_crop = (point.x).abs();
            //Bisogna fare in modo che i numeri passati al crop siano pari altrimenti l'encoder darà problemi
            DimensionToCrop{
                top: make_even_and_convert(top_crop),
                bottom: make_even_and_convert(bottom_crop),
                right: make_even_and_convert(right_crop),
                left: make_even_and_convert(left_crop),
            }
        },
        None => {
            //stream at FullScreen
            DimensionToCrop{
            top: 0,
            bottom: 0,
            right: 0,
            left: 0
        }
    },
    }
}
}

fn make_even_and_convert(num: f32) -> i32 {
    let rounded = num.round() as i32;
    if rounded % 2 != 0 {
        return rounded + 1;
    }
    rounded
}   

fn main() {
    iced::application("Streamer_gui", Streamer::update, Streamer::view)
    .theme(|_| iced::Theme::TokyoNight)
    .subscription(Streamer::subscription)
    .run();
}
