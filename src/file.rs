use crate::LINE_HEIGHT;
use iced::{
    Event, Length, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        Clipboard, Layout, Renderer as _, Shell, Text, Widget,
        layout::{Limits, Node},
        mouse::{self, Click, Cursor},
        renderer::{Quad, Style},
        svg::{Handle, Renderer as _, Svg},
        text::{LineHeight, Renderer as _, Shaping, Wrapping},
        widget::{Tree, tree},
    },
    alignment::{Horizontal, Vertical},
};
use std::path::PathBuf;

const FILE: &[u8] = include_bytes!("../assets/system-uicons--document.svg");

#[derive(Default)]
struct State {
    last_click: Option<Click>,
    hovered: bool,
}

#[derive(Clone)]
pub struct File<Message> {
    path: PathBuf,
    name: String,
    on_single_click: Option<fn(PathBuf) -> Message>,
    on_double_click: Option<fn(PathBuf) -> Message>,
}

impl<Message> File<Message> {
    pub fn new_inner(
        path: PathBuf,
        on_single_click: Option<fn(PathBuf) -> Message>,
        on_double_click: Option<fn(PathBuf) -> Message>,
        show_extensions: bool,
    ) -> Self {
        debug_assert!(path.is_file());

        let name = if show_extensions {
            path.file_name()
        } else {
            path.file_stem()
        }
        .unwrap()
        .to_string_lossy()
        .into_owned();

        Self {
            path,
            name,
            on_single_click,
            on_double_click,
        }
    }
}

impl<Message> Widget<Message, Theme, Renderer> for File<Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(&self, _tree: &mut Tree, _renderer: &Renderer, limits: &Limits) -> Node {
        Node::new(Size::new(limits.max().width, LINE_HEIGHT))
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &Style,
        layout: Layout<'_>,
        _cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        if !bounds.intersects(viewport) {
            return;
        };

        let background = Quad {
            bounds,
            ..Quad::default()
        };

        let state = tree.state.downcast_ref::<State>();

        let background_color = if state.hovered {
            theme.extended_palette().secondary.weak.color
        } else {
            theme.extended_palette().primary.weak.color
        };

        renderer.fill_quad(background, background_color);

        let icon =
            Svg::new(Handle::from_memory(FILE)).color(theme.extended_palette().secondary.base.text);

        renderer.draw_svg(
            icon,
            Rectangle::new(bounds.position(), Size::new(LINE_HEIGHT, LINE_HEIGHT)),
        );

        let name = Text {
            content: self.name.clone(),
            bounds: Size::new(f32::INFINITY, 0.0),
            size: renderer.default_size(),
            line_height: LineHeight::default(),
            font: renderer.default_font(),
            horizontal_alignment: Horizontal::Left,
            vertical_alignment: Vertical::Top,
            shaping: Shaping::Advanced,
            wrapping: Wrapping::None,
        };

        renderer.fill_text(
            name,
            bounds.position() + Vector::new(LINE_HEIGHT, -1.0),
            theme.extended_palette().secondary.base.text,
            bounds,
        );
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let hovered = state.hovered;

        if shell.is_event_captured() {
            state.hovered = false;

            if hovered != state.hovered {
                shell.request_redraw();
            }

            return;
        }

        if let Some(pos) = cursor.position_in(layout.bounds()) {
            state.hovered = true;

            if *event == Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) {
                if let Some(on_single_click) = self.on_single_click {
                    shell.publish(on_single_click(self.path.clone()));
                }

                if let Some(on_double_click) = self.on_double_click {
                    let new_click = Click::new(pos, mouse::Button::Left, state.last_click);

                    if matches!(new_click.kind(), mouse::click::Kind::Double) {
                        shell.publish(on_double_click(self.path.clone()));
                    }

                    state.last_click = Some(new_click);
                }

                shell.capture_event();
            }
        } else {
            state.hovered = false;
        };

        if hovered != state.hovered {
            shell.request_redraw();
        }
    }
}
