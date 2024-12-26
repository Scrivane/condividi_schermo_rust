use iced::Point;

use crate::streamer::streamer::DimensionToCrop;

use super::display::Display;



pub fn dimension_to_crop(first_point: Option<Point>, second_point: Option<Point>, selected_screen: Option<Display>) -> DimensionToCrop
{
    match first_point{
        Some(point) => {
            //calculate the Dimension to stream
            let (x1, y1) = (point.x, point.y);
            let (x2, y2) = (second_point.unwrap().x, second_point.unwrap().y);

            let min_x = x1.min(x2);
            let max_x = x1.max(x2);
            let min_y = y1.min(y2);
            let max_y = y1.max(y2);

            let top_crop = (min_y - Point::ORIGIN.y);
            let bottom_crop = (selected_screen.unwrap().height as f32 - max_y);
            let right_crop = (selected_screen.unwrap().width as f32 - max_x);
            let left_crop = (min_x);
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


fn make_even_and_convert(num: f32) -> i32 {
    let rounded = num.round() as i32;

    
    if rounded % 2 != 0 {
        println!("ROUNDED: {}", rounded);
        return rounded + 1;
    }
    println!("ROUNDED: {}", rounded);
    rounded
}