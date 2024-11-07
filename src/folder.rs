use crate::{errentry::ErrEntry, file::File};
use iced::{
    advanced::{
        graphics::geometry::Renderer as _,
        layout::{self, flex::Axis, Limits, Node},
        mouse::{self, Cursor},
        renderer::{Quad, Style},
        svg::{Handle, Renderer as _, Svg},
        text::{LineHeight, Renderer as _, Shaping, Wrapping},
        widget::{tree, Tree},
        Clipboard, Layout, Renderer as _, Shell, Text, Widget,
    },
    alignment::{Horizontal, Vertical},
    event::Status,
    widget::canvas::{Frame, Path, Stroke},
    Alignment, Element, Event, Length, Padding, Point, Rectangle, Renderer, Size, Theme, Vector,
};
use std::{
    cell::{OnceCell, RefCell},
    cmp::Ordering,
    fmt::{Debug, Formatter},
    ops::Deref,
    path::PathBuf,
    rc::Rc,
};

static FOLDER_CLOSED: &[u8] = include_bytes!("../assets/system-uicons--chevron-right.svg");
static FOLDER_OPEN: &[u8] = include_bytes!("../assets/system-uicons--chevron-down.svg");

#[derive(Default)]
struct State {
    open: bool,
    line_height: OnceCell<f32>,
}

#[expect(clippy::type_complexity)]
pub struct Folder<'a, Message> {
    path: PathBuf,
    name: String,
    children: OnceCell<Vec<Element<'a, Message, Theme, Renderer>>>,
    on_single_click: Rc<RefCell<Option<Box<dyn Fn(PathBuf) -> Message + 'a>>>>,
    on_double_click: Rc<RefCell<Option<Box<dyn Fn(PathBuf) -> Message + 'a>>>>,
    show_hidden: bool,
}

impl Debug for Folder<'_, ()> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Folder")
            .field("path", &self.path)
            .field("show_hidden", &self.show_hidden)
            .finish_non_exhaustive()
    }
}

impl<'a, Message> Folder<'a, Message>
where
    Message: Clone + 'a,
{
    #[must_use]
    pub fn new(path: PathBuf) -> Option<Self> {
        if std::fs::read_dir(&path).is_err() {
            return None;
        }

        let name = path.file_name()?.to_string_lossy().into_owned();

        Some(Self {
            path,
            name,
            children: OnceCell::new(),
            on_single_click: Rc::default(),
            on_double_click: Rc::default(),
            show_hidden: false,
        })
    }

    #[must_use]
    pub fn on_single_click(self, on_single_click: impl Fn(PathBuf) -> Message + 'a) -> Self {
        self.on_single_click
            .borrow_mut()
            .replace(Box::new(on_single_click));
        self
    }

    #[must_use]
    pub fn on_double_click(self, on_double_click: impl Fn(PathBuf) -> Message + 'a) -> Self {
        self.on_double_click
            .borrow_mut()
            .replace(Box::new(on_double_click));
        self
    }

    #[must_use]
    pub fn hidden(mut self, show_hidden: bool) -> Self {
        self.show_hidden = show_hidden;
        self
    }

    #[expect(clippy::type_complexity)]
    fn new_inner(
        path: PathBuf,
        on_single_click: Rc<RefCell<Option<Box<dyn Fn(PathBuf) -> Message + 'a>>>>,
        on_double_click: Rc<RefCell<Option<Box<dyn Fn(PathBuf) -> Message + 'a>>>>,
        show_hidden: bool,
    ) -> Option<Self> {
        if std::fs::read_dir(&path).is_err() {
            return None;
        }

        let name = path.file_name().unwrap().to_string_lossy().into_owned();

        Some(Self {
            path,
            name,
            children: OnceCell::new(),
            on_single_click,
            on_double_click,
            show_hidden,
        })
    }

    fn init_children(&self) -> Vec<Element<'a, Message, Theme, Renderer>> {
        let mut files: Vec<_> = std::fs::read_dir(&self.path)
            .unwrap()
            .filter_map(Result::ok)
            .map(|file| {
                let mut name = file.file_name();
                name.make_ascii_lowercase();

                (file, name)
            })
            .filter(|(_, name)| !self.show_hidden && !name.as_encoded_bytes().starts_with(b"."))
            .collect();

        files.sort_by(|(a, aname), (b, bname)| {
            if let (Ok(a), Ok(b)) = (a.file_type(), b.file_type()) {
                if !a.is_dir() && b.is_dir() {
                    return Ordering::Greater;
                } else if a.is_dir() && !b.is_dir() {
                    return Ordering::Less;
                }
            }

            aname.cmp(bname)
        });

        files
            .into_iter()
            .map(|(entry, _)| {
                let path = entry.path();
                if path.is_file() {
                    let file = File::new_inner(
                        path,
                        self.on_single_click.clone(),
                        self.on_double_click.clone(),
                    )
                    .into();
                    file
                } else {
                    let Some(folder) = Folder::new_inner(
                        path.clone(),
                        self.on_single_click.clone(),
                        self.on_double_click.clone(),
                        self.show_hidden,
                    ) else {
                        return ErrEntry::new_inner(&path).into();
                    };

                    folder.into()
                }
            })
            .collect()
    }
}

impl<'a, Message> Widget<Message, Theme, Renderer> for Folder<'a, Message>
where
    Message: Clone + 'a,
{
    fn children(&self) -> Vec<Tree> {
        self.children
            .get()
            .unwrap_or(&vec![])
            .iter()
            .map(Tree::new)
            .collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_ref::<State>();

        tree.diff_children(self.children.get().map_or_else(
            || {
                if state.open {
                    &**self.children.get_or_init(|| self.init_children())
                } else {
                    &[]
                }
            },
            Deref::deref,
        ));
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let state = tree.state.downcast_ref::<State>();

        if !state.open {
            return Node::new(Size::new(
                limits.max().width,
                *state
                    .line_height
                    .get_or_init(|| (renderer.default_size().0 * 1.3).ceil()),
            ));
        }

        layout::flex::resolve(
            Axis::Vertical,
            renderer,
            limits,
            Length::Fill,
            Length::Shrink,
            Padding::ZERO
                .top(*state.line_height.get().unwrap())
                .left(*state.line_height.get().unwrap()),
            0.0,
            Alignment::Start,
            self.children.get_or_init(|| self.init_children()),
            &mut tree.children,
        )
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> Status {
        let state = tree.state.downcast_mut::<State>();

        if let Some(pos) = cursor.position() {
            if event == Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                && layout.bounds().contains(pos)
                && &cursor.position_in(layout.bounds()).unwrap().y
                    <= state.line_height.get().unwrap()
            {
                state.open ^= true;
                self.children.get_or_init(|| self.init_children());
                self.diff(tree);
                shell.invalidate_layout();
                return Status::Captured;
            }
        }

        if !state.open {
            return Status::Ignored;
        }

        self.children
            .get_mut()
            .unwrap()
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child.as_widget_mut().on_event(
                    state,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(Status::Ignored, Status::merge)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let background = Quad {
            bounds: Rectangle::new(
                bounds.position(),
                Size::new(bounds.width, *state.line_height.get().unwrap()),
            ),
            ..Quad::default()
        };
        let background_color = cursor.position_in(bounds).map_or_else(
            || theme.extended_palette().primary.weak.color,
            |pos| {
                if &pos.y <= state.line_height.get().unwrap() {
                    theme.extended_palette().secondary.weak.color
                } else {
                    theme.extended_palette().primary.weak.color
                }
            },
        );

        renderer.fill_quad(background, background_color);

        let icon = Svg::new(Handle::from_memory(if state.open {
            FOLDER_OPEN
        } else {
            FOLDER_CLOSED
        }))
        .color(theme.extended_palette().secondary.base.text);

        let offset = (state.line_height.get().unwrap() * 0.1).round();
        renderer.draw_svg(
            icon,
            Rectangle::new(
                bounds.position() + Vector::new(-offset, -offset),
                Size::new(
                    state.line_height.get().unwrap() + 2.0 * offset,
                    state.line_height.get().unwrap() + 2.0 * offset,
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

        if state.open {
            self.children
                .get()
                .unwrap()
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
                .filter(|(_, layout)| layout.bounds().intersects(viewport))
                .for_each(|((child, tree), layout)| {
                    child
                        .as_widget()
                        .draw(tree, renderer, theme, style, layout, cursor, viewport);
                });

            let mut frame = Frame::new(renderer, bounds.size());

            let path = Path::new(|path| {
                path.line_to(Point::new(
                    state.line_height.get().unwrap() * 0.5,
                    state.line_height.get().unwrap() * 1.5,
                ));
                path.line_to(Point::new(
                    state.line_height.get().unwrap() / 2.0,
                    bounds.size().height - state.line_height.get().unwrap() / 2.0,
                ));
            });

            frame.with_clip(bounds, |frame| {
                frame.stroke(
                    &path,
                    Stroke::default()
                        .with_color(theme.extended_palette().primary.weak.color)
                        .with_width(2.0),
                );
            });

            #[expect(clippy::unit_arg, reason = "clippy bug")]
            renderer.draw_geometry(frame.into_geometry());
        }
    }
}

impl<'a, Message> From<Folder<'a, Message>> for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
{
    fn from(folder: Folder<'a, Message>) -> Self {
        Self::new(folder)
    }
}
