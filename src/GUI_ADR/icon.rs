use iced::{
    advanced::{
        layout, mouse,
        renderer::{self, Quad},
        widget::Tree,
        Layout, Widget,
    }, color, widget::{button, image::Handle, Theme}, Border, Color, Element, Length, Rectangle, Shadow, Size
};

pub struct Icon {
    handle: Handle,
}

impl Icon {
   pub fn new(icon_handle: Handle) -> Self {
        Self {
            handle: icon_handle,
        }
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for Icon
where
    Renderer: iced::advanced::Renderer + iced::advanced::image::Renderer<Handle = Handle>,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        iced::widget::image::layout(
            renderer,
            limits,
            &self.handle,
            Length::Fixed(75.),
            Length::Fixed(75.),
            iced::ContentFit::Contain,
            iced::Rotation::Solid(0.into())
        )
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        renderer.fill_quad(
            Quad {
                bounds: layout.bounds(),
                border: Border {
                    color: color!(0x1a1b26),
                    width: 1.0,
                    radius: 10.0.into(),
                },
                shadow: Shadow::default(),
            },
            Color::TRANSPARENT,
        );

        iced::widget::image::draw(
            renderer,
            layout,
            &self.handle,
            iced::ContentFit::Contain,
            iced::widget::image::FilterMethod::Linear,
            iced::Rotation::Solid(0.into()),
            0.5
        );
    }
}

impl<'a, Message, Renderer> From<Icon> for Element<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer + iced::advanced::image::Renderer<Handle = Handle>,
{
    fn from(widget: Icon) -> Self {
        Self::new(widget)
    }
}
