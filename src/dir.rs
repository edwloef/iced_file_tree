use crate::{file::File, LINE_HEIGHT};
use iced::{
    advanced::{
        layout::{Limits, Node},
        mouse::{self, Cursor},
        renderer::{Quad, Style},
        svg::{Handle, Renderer as _, Svg},
        text::{LineHeight, Renderer as _, Shaping, Wrapping},
        widget::{tree, Tree},
        Clipboard, Layout, Renderer as _, Shell, Text, Widget,
    },
    alignment::{Horizontal, Vertical},
    event::Status,
    Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector,
};
use std::{
    cell::{OnceCell, RefCell},
    ops::Deref,
    path::PathBuf,
    rc::Rc,
};

const DIR_CLOSED: &[u8] = include_bytes!("../assets/system-uicons--chevron-right.svg");
const DIR_OPEN: &[u8] = include_bytes!("../assets/system-uicons--chevron-down.svg");

struct State<Message> {
    open: bool,
    dirs: OnceCell<Rc<[Dir<Message>]>>,
    files: OnceCell<Rc<[File<Message>]>>,
}

impl<Message> Default for State<Message> {
    fn default() -> Self {
        Self {
            open: false,
            dirs: OnceCell::new(),
            files: OnceCell::new(),
        }
    }
}

#[expect(clippy::type_complexity)]
#[derive(Clone)]
pub struct Dir<Message> {
    pub path: PathBuf,
    name: String,
    dirs: OnceCell<Rc<[Dir<Message>]>>,
    files: OnceCell<Rc<[File<Message>]>>,
    pub on_single_click: Rc<RefCell<Option<Box<dyn Fn(PathBuf) -> Message>>>>,
    pub on_double_click: Rc<RefCell<Option<Box<dyn Fn(PathBuf) -> Message>>>>,
    pub show_hidden: bool,
    pub show_extensions: bool,
}

impl<Message> Dir<Message>
where
    Message: Clone + 'static,
{
    #[expect(clippy::type_complexity)]
    pub fn new_inner(
        path: PathBuf,
        on_single_click: Rc<RefCell<Option<Box<dyn Fn(PathBuf) -> Message>>>>,
        on_double_click: Rc<RefCell<Option<Box<dyn Fn(PathBuf) -> Message>>>>,
        show_hidden: bool,
        show_extensions: bool,
    ) -> Self {
        debug_assert!(path.is_dir());

        let name = path.file_name().unwrap().to_string_lossy().into_owned();

        Self {
            path,
            name,
            files: OnceCell::default(),
            dirs: OnceCell::default(),
            on_single_click,
            on_double_click,
            show_hidden,
            show_extensions,
        }
    }

    fn init_children(&self, state: &State<Message>) -> Vec<Element<'_, Message, Theme, Renderer>> {
        if !state.open {
            return vec![];
        }

        self.dirs
            .get_or_init(|| state.dirs.get_or_init(|| self.init_dirs()).clone())
            .iter()
            .cloned()
            .map(Element::new)
            .chain(
                self.files
                    .get_or_init(|| state.files.get_or_init(|| self.init_files()).clone())
                    .iter()
                    .cloned()
                    .map(Element::new),
            )
            .collect()
    }

    fn get_children(&self) -> Vec<Element<'_, Message, Theme, Renderer>> {
        self.dirs
            .get()
            .into_iter()
            .flat_map(Deref::deref)
            .cloned()
            .map(Element::new)
            .chain(
                self.files
                    .get()
                    .into_iter()
                    .flat_map(Deref::deref)
                    .cloned()
                    .map(Element::new),
            )
            .collect()
    }

    fn init_files(&self) -> Rc<[File<Message>]> {
        let Ok(files) = std::fs::read_dir(&self.path) else {
            return [].into();
        };

        let mut files: Box<_> = files
            .filter_map(Result::ok)
            .filter(|file| file.file_type().is_ok_and(|t| t.is_file()))
            .map(|file| {
                let mut name = file.file_name();
                name.make_ascii_lowercase();

                (file, name)
            })
            .filter(|(_, name)| !self.show_hidden && !name.as_encoded_bytes().starts_with(b"."))
            .collect();
        files.sort_by(|(_, aname), (_, bname)| aname.cmp(bname));
        files
            .iter()
            .map(|(entry, _)| {
                let path = entry.path();
                File::new_inner(
                    path,
                    self.on_single_click.clone(),
                    self.on_double_click.clone(),
                    self.show_extensions,
                )
            })
            .collect()
    }

    fn init_dirs(&self) -> Rc<[Self]> {
        let Ok(dirs) = std::fs::read_dir(&self.path) else {
            return [].into();
        };

        let mut dirs: Box<_> = dirs
            .filter_map(Result::ok)
            .filter(|file| file.file_type().is_ok_and(|t| t.is_dir()))
            .map(|file| {
                let mut name = file.file_name();
                name.make_ascii_lowercase();

                (file, name)
            })
            .filter(|(_, name)| !self.show_hidden && !name.as_encoded_bytes().starts_with(b"."))
            .collect();
        dirs.sort_by(|(_, aname), (_, bname)| aname.cmp(bname));
        dirs.iter()
            .map(|(entry, _)| {
                let path = entry.path();
                Self::new_inner(
                    path,
                    self.on_single_click.clone(),
                    self.on_double_click.clone(),
                    self.show_hidden,
                    self.show_extensions,
                )
            })
            .collect()
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Dir<Message>
where
    Message: Clone + 'static,
{
    fn children(&self) -> Vec<Tree> {
        self.get_children().iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_ref::<State<Message>>();

        tree.diff_children(&self.init_children(state));
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Message>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Message>::default())
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let state = tree.state.downcast_ref::<State<Message>>();

        if !state.open {
            return Node::new(Size::new(limits.max().width, LINE_HEIGHT));
        }

        self.diff(tree);

        let x = LINE_HEIGHT;
        let mut y = LINE_HEIGHT;

        let children = self
            .get_children()
            .iter()
            .zip(&mut tree.children)
            .map(|(child, tree)| child.as_widget().layout(tree, renderer, limits))
            .map(|layout| {
                let layout = layout.translate(Vector::new(x, y));
                y += layout.size().height;
                layout
            })
            .collect();

        Node::with_children(Size::new(limits.max().width, y), children)
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
        let state = tree.state.downcast_mut::<State<Message>>();

        if event == Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            && cursor
                .position_in(layout.bounds())
                .is_some_and(|p| p.y <= LINE_HEIGHT)
        {
            state.open ^= true;
            shell.invalidate_layout();
            return Status::Captured;
        }

        if !state.open {
            return Status::Ignored;
        }

        self.init_children(state)
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
        let Some(bounds) = layout.bounds().intersection(viewport) else {
            return;
        };

        let state = tree.state.downcast_ref::<State<Message>>();

        let background = Quad {
            bounds: Rectangle::new(bounds.position(), Size::new(bounds.width, LINE_HEIGHT)),
            ..Quad::default()
        };
        let background_color = cursor.position_in(bounds).map_or_else(
            || theme.extended_palette().primary.weak.color,
            |pos| {
                if pos.y <= LINE_HEIGHT {
                    theme.extended_palette().secondary.weak.color
                } else {
                    theme.extended_palette().primary.weak.color
                }
            },
        );

        renderer.fill_quad(background, background_color);

        let icon = Svg::new(Handle::from_memory(if state.open {
            DIR_OPEN
        } else {
            DIR_CLOSED
        }))
        .color(theme.extended_palette().secondary.base.text);

        renderer.draw_svg(
            icon,
            Rectangle::new(
                bounds.position() + Vector::new(-2.0, -2.0),
                Size::new(LINE_HEIGHT + 4.0, LINE_HEIGHT + 4.0),
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
            bounds.position() + Vector::new(LINE_HEIGHT, -1.0),
            theme.extended_palette().secondary.base.text,
            bounds,
        );

        if state.open && !self.init_children(state).is_empty() {
            self.get_children()
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
                .filter(|(_, layout)| layout.bounds().intersects(viewport))
                .for_each(|((child, tree), layout)| {
                    child
                        .as_widget()
                        .draw(tree, renderer, theme, style, layout, cursor, viewport);
                });

            let offset = Vector::new(
                LINE_HEIGHT.mul_add(0.5, -1.0),
                LINE_HEIGHT.mul_add(1.5, -1.0),
            );
            let size = Size::new(2.0, bounds.size().height - offset.x - offset.y);
            let line = Quad {
                bounds: Rectangle::new(bounds.position() + offset, size),
                ..Default::default()
            };

            renderer.fill_quad(line, theme.extended_palette().primary.weak.color);
        }
    }
}
