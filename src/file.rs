use core::f32;
use iced::{
    advanced::{
        layout::{Limits, Node},
        mouse::{self, Click, Cursor},
        renderer::{Quad, Style},
        svg::{Handle, Renderer as _, Svg},
        text::{LineHeight, Renderer as _, Shaping, Wrapping},
        widget::{tree, Tree},
        Layout, Renderer as _, Text, Widget,
    },
    alignment::{Horizontal, Vertical},
    event::Status,
    Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector,
};
use std::{
    cell::{OnceCell, RefCell},
    fmt::{Debug, Formatter},
    path::PathBuf,
    rc::Rc,
};

const FILE: &[u8] = include_bytes!("../assets/system-uicons--document.svg");

#[derive(Default)]
struct State {
    last_click: Option<Click>,
    line_height: OnceCell<f32>,
}

#[expect(clippy::type_complexity)]
pub struct File<'a, Message> {
    path: PathBuf,
    name: String,
    on_single_click: Rc<RefCell<Option<Box<dyn Fn(PathBuf) -> Message + 'a>>>>,
    on_double_click: Rc<RefCell<Option<Box<dyn Fn(PathBuf) -> Message + 'a>>>>,
}

impl Debug for File<'_, ()> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("File")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl<'a, Message> File<'a, Message> {
    #[expect(clippy::type_complexity)]
    pub fn new_inner(
        path: PathBuf,
        on_single_click: Rc<RefCell<Option<Box<dyn Fn(PathBuf) -> Message + 'a>>>>,
        on_double_click: Rc<RefCell<Option<Box<dyn Fn(PathBuf) -> Message + 'a>>>>,
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

impl<Message> Widget<Message, Theme, Renderer> for File<'_, Message> {
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
        cursor: Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let background = Quad {
            bounds,
            ..Quad::default()
        };
        let background_color = cursor.position_in(bounds).map_or_else(
            || theme.extended_palette().primary.weak.color,
            |_| theme.extended_palette().secondary.weak.color,
        );

        renderer.fill_quad(background, background_color);

        let icon =
            Svg::new(Handle::from_memory(FILE)).color(theme.extended_palette().secondary.base.text);

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
            content: self.name.clone(),
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
            theme.extended_palette().secondary.base.text,
            bounds,
        );
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> Status {
        let state = tree.state.downcast_mut::<State>();

        if let Some(pos) = cursor.position() {
            if event == Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                && layout.bounds().contains(pos)
            {
                if let Some(on_single_click) = self.on_single_click.borrow().as_deref() {
                    shell.publish(on_single_click(self.path.clone()));
                }

                if let Some(on_double_click) = self.on_double_click.borrow().as_deref() {
                    let new_click = Click::new(pos, mouse::Button::Left, state.last_click);

                    if matches!(new_click.kind(), mouse::click::Kind::Double) {
                        shell.publish(on_double_click(self.path.clone()));
                    }

                    state.last_click = Some(new_click);
                }

                return Status::Captured;
            }
        }

        Status::Ignored
    }
}

impl<'a, Message> From<File<'a, Message>> for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
{
    fn from(file: File<'a, Message>) -> Self {
        Self::new(file)
    }
}
