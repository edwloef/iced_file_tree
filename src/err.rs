use core::f32;
use iced::{
    advanced::{
        layout::{Limits, Node},
        mouse::Cursor,
        renderer::{Quad, Style},
        svg::{Handle, Renderer as _, Svg},
        text::{LineHeight, Renderer as _, Shaping, Wrapping},
        widget::{tree, Tree},
        Layout, Renderer as _, Text, Widget,
    },
    alignment::{Horizontal, Vertical},
    Element, Length, Rectangle, Renderer, Size, Theme, Vector,
};
use std::{cell::OnceCell, path::PathBuf};

static ERROR: &[u8] = include_bytes!("../assets/system-uicons--cross.svg");

#[derive(Default)]
struct State {
    line_height: OnceCell<f32>,
}

#[expect(missing_debug_implementations)]
pub struct ErrorFile {
    path: PathBuf,
}

impl ErrorFile {
    pub fn new_inner(path: PathBuf) -> Self {
        Self { path }
    }
}

impl<Message> Widget<Message, Theme, Renderer> for ErrorFile {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let state = tree.state.downcast_ref::<State>();

        Node::new(Size::new(
            limits.max().width,
            *state
                .line_height
                .get_or_init(|| (renderer.default_size().0 * 1.3).ceil()),
        ))
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let background = Quad {
            bounds,
            ..Quad::default()
        };
        let background_color = theme.extended_palette().danger.weak.color;

        renderer.fill_quad(background, background_color);

        let icon = Svg::new(Handle::from_memory(ERROR))
            .color(theme.extended_palette().danger.strong.color);

        renderer.draw_svg(
            icon,
            Rectangle::new(
                bounds.position(),
                Size::new(
                    *state.line_height.get().unwrap(),
                    *state.line_height.get().unwrap(),
                ),
            ),
        );

        let name = Text {
            content: self
                .path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
            bounds: Size::new(f32::INFINITY, 0.0),
            size: renderer.default_size(),
            line_height: LineHeight::default(),
            font: renderer.default_font(),
            horizontal_alignment: Horizontal::Left,
            vertical_alignment: Vertical::Top,
            shaping: Shaping::default(),
            wrapping: Wrapping::default(),
        };

        renderer.fill_text(
            name,
            bounds.position() + Vector::new(*state.line_height.get().unwrap(), 0.0),
            theme.extended_palette().danger.strong.color,
            bounds,
        );
    }
}

impl<Message> From<ErrorFile> for Element<'_, Message, Theme, Renderer> {
    fn from(file: ErrorFile) -> Self {
        Self::new(file)
    }
}
