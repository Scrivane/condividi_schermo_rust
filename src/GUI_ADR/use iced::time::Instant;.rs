use iced::time::Instant;
use iced::widget::canvas::{Frame, Geometry, Program};
use iced::widget::{
    self, center, checkbox, column, container, image, pick_list, row, slider, text, Canvas, Container
};


use iced::{window, Point, event::{self, Event, Status},
executor,
mouse::{self, Event::{ButtonPressed, ButtonReleased, CursorMoved}},
touch::Event::FingerMoved};
use iced::{
    Bottom, Center, Color, ContentFit, Degrees, Element, Fill, Length, Radians, Rectangle, Renderer, Rotation, Subscription, Theme
};

use screenshots::Screen;


pub fn main() -> iced::Result {
    let screens = Screen::all().unwrap();
    
    for screen in screens {
        println!("capturer {screen:?}");
        let mut image = screen.capture().unwrap();
        image
            .save(format!("target/{}.png", screen.display_info.id))
            .unwrap();

    }
        iced::application("Ferris - Iced", Image::update, Image::view)
        .subscription(Image::subscription)
        .theme(|_| Theme::TokyoNight)
        .run()
}

struct Image {
    width: f32,
    height: f32,
    mouse_point: Point,
    first_point: Option<Point>,
    second_point: Option<Point>,
    is_selecting_area: bool
}

#[derive(Debug, Clone, Copy)]
enum Message {
    WidthChanged(f32),
    HeightChanged(f32),
    PointUpdated(Point),
    FirstPoint,
    SecondPoint,
    ToggleSelectingArea,
}

impl Image {
    fn update(&mut self, message: Message) {
        match message {
            Message::WidthChanged(width) => {
                self.width = width;
            },
            Message::HeightChanged(height)=>{
                self.height = height;
            },
            Message::PointUpdated(p) => self.mouse_point = p,
            Message::FirstPoint => {
                self.first_point = Some(self.mouse_point)
            },
            Message::SecondPoint => {
                self.second_point = Some(self.mouse_point);
                println!("New Points saved: {}, {}", self.first_point.unwrap(), self.second_point.unwrap());
               // self.is_selecting_area = false;
            },
            Message::ToggleSelectingArea => {
                self.first_point = Some(Point::ORIGIN);
                self.is_selecting_area = true;

            }
        }
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
        let column = column![
            Element::from(
                image("target/1.png")
                .width(Length::Fill)
                .height(Length::Fill)
                .content_fit(ContentFit::Cover)
            )
        ];

        let over_text = text("Choose the area to stream")
        .color(Color::WHITE);

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
        .height(Length::Fill);
                
        return my_container.into();
           
    }
}

impl Default for Image {
    fn default() -> Self {
        let screens = Screen::all().unwrap();
        let initial_height =  screens[0].display_info.height;
        let initial_width =  screens[0].display_info.width;
        
        Self {
            width: initial_width as f32,
            height :initial_height as f32,
            mouse_point: Point::ORIGIN,
            first_point: None,
            second_point: None,
            is_selecting_area: true
        }
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