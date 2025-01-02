use iced::{mouse, widget::canvas::{Frame, Geometry, Program}, Color, Point, Rectangle, Renderer, Theme};


pub struct MyCanvas{
    pub first_point: Option<Point>,
    pub second_point: Point,
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