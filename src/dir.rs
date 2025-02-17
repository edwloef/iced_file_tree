use crate::{LINE_HEIGHT, file::File};
use iced::{
    Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector,
    advanced::{
        Clipboard, Layout, Renderer as _, Shell, Text, Widget,
        layout::{Limits, Node},
        mouse::{self, Cursor},
        renderer::{Quad, Style},
        svg::{Handle, Renderer as _, Svg},
        text::{LineHeight, Renderer as _, Shaping, Wrapping},
        widget::{Tree, tree},
    },
    alignment::{Horizontal, Vertical},
};
use std::{cell::OnceCell, ops::Deref, path::PathBuf, rc::Rc};

const DIR_CLOSED: &[u8] = include_bytes!("../assets/system-uicons--chevron-right.svg");
const DIR_OPEN: &[u8] = include_bytes!("../assets/system-uicons--chevron-down.svg");

struct State<Message> {
    open: bool,
    hovered: bool,
    dirs: OnceCell<Rc<[Dir<Message>]>>,
    files: OnceCell<Rc<[File<Message>]>>,
}

impl<Message> Default for State<Message> {
    fn default() -> Self {
        Self {
            open: bool::default(),
            hovered: bool::default(),
            dirs: OnceCell::default(),
            files: OnceCell::default(),
        }
    }
}

#[derive(Clone)]
pub struct Dir<Message> {
    pub path: PathBuf,
    name: String,
    dirs: OnceCell<Rc<[Dir<Message>]>>,
    files: OnceCell<Rc<[File<Message>]>>,
    pub on_single_click: Option<fn(PathBuf) -> Message>,
    pub on_double_click: Option<fn(PathBuf) -> Message>,
    pub show_hidden: bool,
    pub show_extensions: bool,
}

impl<Message> Dir<Message>
where
    Message: Clone + 'static,
{
    pub fn new_inner(
        path: PathBuf,
        on_single_click: Option<fn(PathBuf) -> Message>,
        on_double_click: Option<fn(PathBuf) -> Message>,
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

    fn init_children(
        &self,
        state: &State<Message>,
    ) -> impl Iterator<Item = Element<'_, Message, Theme, Renderer>> + use<'_, Message> {
        let dirs = if state.open {
            &**self
                .dirs
                .get_or_init(|| state.dirs.get_or_init(|| self.init_dirs()).clone())
        } else {
            &[]
        };

        let files = if state.open {
            &**self
                .files
                .get_or_init(|| state.files.get_or_init(|| self.init_files()).clone())
        } else {
            &[]
        };

        dirs.iter()
            .cloned()
            .map(Element::new)
            .chain(files.iter().cloned().map(Element::new))
    }

    fn get_children(&self) -> impl Iterator<Item = Element<'_, Message, Theme, Renderer>> {
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
    }

    fn init_files(&self) -> Rc<[File<Message>]> {
        let Ok(files) = std::fs::read_dir(&self.path) else {
            return [].into();
        };

        let mut files = files
            .filter_map(Result::ok)
            .filter(|file| file.file_type().is_ok_and(|t| t.is_file()))
            .map(|file| {
                let mut name = file.file_name();
                name.make_ascii_lowercase();

                (file, name)
            })
            .filter(|(_, name)| !self.show_hidden && !name.as_encoded_bytes().starts_with(b"."))
            .collect::<Box<_>>();
        files.sort_by(|(_, aname), (_, bname)| aname.cmp(bname));
        files
            .iter()
            .map(|(entry, _)| {
                let path = entry.path();
                File::new_inner(
                    path,
                    self.on_single_click,
                    self.on_double_click,
                    self.show_extensions,
                )
            })
            .collect()
    }

    fn init_dirs(&self) -> Rc<[Self]> {
        let Ok(dirs) = std::fs::read_dir(&self.path) else {
            return [].into();
        };

        let mut dirs = dirs
            .filter_map(Result::ok)
            .filter(|file| file.file_type().is_ok_and(|t| t.is_dir()))
            .map(|file| {
                let mut name = file.file_name();
                name.make_ascii_lowercase();

                (file, name)
            })
            .filter(|(_, name)| !self.show_hidden && !name.as_encoded_bytes().starts_with(b"."))
            .collect::<Box<_>>();
        dirs.sort_by(|(_, aname), (_, bname)| aname.cmp(bname));
        dirs.iter()
            .map(|(entry, _)| {
                let path = entry.path();
                Self::new_inner(
                    path,
                    self.on_single_click,
                    self.on_double_click,
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
        self.get_children().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_ref::<State<Message>>();

        tree.diff_children(&self.init_children(state).collect::<Box<_>>());
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

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State<Message>>();
        let hovered = state.hovered;

        if shell.is_event_captured() {
            state.hovered = false;

            if hovered != state.hovered {
                shell.request_redraw();
            }

            return;
        }

        if cursor
            .position_in(layout.bounds())
            .is_some_and(|p| p.y <= LINE_HEIGHT)
        {
            state.hovered = true;

            if *event == Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) {
                state.open ^= true;

                shell.invalidate_layout();
                shell.request_redraw();
                shell.capture_event();
            }
        } else {
            state.hovered = false;
        }

        if hovered != state.hovered {
            shell.request_redraw();
        }

        if state.open {
            self.init_children(state)
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((mut child, state), layout)| {
                    child.as_widget_mut().update(
                        state, event, layout, cursor, renderer, clipboard, shell, viewport,
                    );
                });
        }
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
        let bounds = layout.bounds();

        if !bounds.intersects(viewport) {
            return;
        };

        let state = tree.state.downcast_ref::<State<Message>>();

        let background = Quad {
            bounds: Rectangle::new(bounds.position(), Size::new(bounds.width, LINE_HEIGHT)),
            ..Quad::default()
        };
        let background_color = if state.hovered {
            theme.extended_palette().secondary.weak.color
        } else {
            theme.extended_palette().primary.weak.color
        };

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
            shaping: Shaping::Advanced,
            wrapping: Wrapping::None,
        };

        renderer.fill_text(
            name,
            bounds.position() + Vector::new(LINE_HEIGHT, -1.0),
            theme.extended_palette().secondary.base.text,
            bounds,
        );

        if state.open && self.init_children(state).next().is_some() {
            self.get_children()
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
