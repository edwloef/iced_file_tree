use crate::file::File;
use iced::{
    advanced::{
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
    Alignment, Element, Event, Length, Padding, Rectangle, Renderer, Size, Theme, Vector,
};
use std::{
    cell::{OnceCell, RefCell},
    fmt::{Debug, Formatter},
    path::PathBuf,
    rc::Rc,
};

const FOLDER_CLOSED: &[u8] = include_bytes!("../assets/system-uicons--chevron-right.svg");
const FOLDER_OPEN: &[u8] = include_bytes!("../assets/system-uicons--chevron-down.svg");

struct State<Message> {
    open: bool,
    line_height: OnceCell<f32>,
    folders: OnceCell<Rc<[Folder<Message>]>>,
    files: OnceCell<Rc<[File<Message>]>>,
}

impl<Message> Default for State<Message> {
    fn default() -> Self {
        Self {
            open: false,
            line_height: OnceCell::new(),
            folders: OnceCell::new(),
            files: OnceCell::new(),
        }
    }
}

/// A lightweight file tree widget for the [iced](https://github.com/iced-rs/iced/tree/master) toolkit.
///
/// # Example
/// ```no_run
/// use iced::widget::scrollable;
/// use iced_file_tree::file_tree;
///
/// enum Message {
///     FileTreeMessage(PathBuf),
///     // ...
/// }
///
/// fn view(state: &State) -> Element<'_, Message> {
///     let path: PathBuf = /* */
///
///     scrollable(
///         file_tree(path)
///             .unwrap()
///             .on_double_click(Message::FileTreeMessage),
///     )
///     .into()
/// }
/// ```
#[expect(clippy::type_complexity)]
#[derive(Clone)]
pub struct Folder<Message> {
    path: PathBuf,
    name: String,
    folders: OnceCell<Rc<[Folder<Message>]>>,
    files: OnceCell<Rc<[File<Message>]>>,
    empty_folders: Rc<[Self]>,
    empty_files: Rc<[File<Message>]>,
    on_single_click: Rc<RefCell<Option<Box<dyn Fn(PathBuf) -> Message>>>>,
    on_double_click: Rc<RefCell<Option<Box<dyn Fn(PathBuf) -> Message>>>>,
    show_hidden: bool,
    show_extensions: bool,
}

impl<Message> Debug for Folder<Message> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Folder")
            .field("path", &self.path)
            .field("show_hidden", &self.show_hidden)
            .finish_non_exhaustive()
    }
}

impl<Message> Folder<Message>
where
    Message: Clone + 'static,
{
    /// Creates a new [`FileTree`](crate::FileTree) with the root at the given path.
    #[must_use]
    pub fn new(path: PathBuf) -> Option<Self> {
        if std::fs::read_dir(&path).is_err() {
            return None;
        }

        let name = path.file_name()?.to_string_lossy().into_owned();

        Some(Self {
            path,
            name,
            files: OnceCell::default(),
            folders: OnceCell::default(),
            empty_folders: [].into(),
            empty_files: [].into(),
            on_single_click: Rc::default(),
            on_double_click: Rc::default(),
            show_hidden: false,
            show_extensions: true,
        })
    }

    /// Sets the message that will be produced when the user single-clicks on a file within the file tree.
    #[must_use]
    pub fn on_single_click(self, on_single_click: impl Fn(PathBuf) -> Message + 'static) -> Self {
        self.on_single_click
            .borrow_mut()
            .replace(Box::new(on_single_click));
        self
    }

    /// Sets the message that will be produced when the user double-clicks on a file within the file tree.
    #[must_use]
    pub fn on_double_click(self, on_double_click: impl Fn(PathBuf) -> Message + 'static) -> Self {
        self.on_double_click
            .borrow_mut()
            .replace(Box::new(on_double_click));
        self
    }

    /// Enables or disables showing hidden files (disabled by default).
    #[must_use]
    pub fn hidden_files(mut self, show_hidden: bool) -> Self {
        self.show_hidden = show_hidden;
        self
    }

    #[must_use]
    /// Enables or disables showing file extensions (enabled by default).
    pub fn file_extensions(mut self, show_extensions: bool) -> Self {
        self.show_extensions = show_extensions;
        self
    }

    #[expect(clippy::type_complexity)]
    fn new_inner(
        path: PathBuf,
        empty_files: Rc<[File<Message>]>,
        empty_folders: Rc<[Self]>,
        on_single_click: Rc<RefCell<Option<Box<dyn Fn(PathBuf) -> Message>>>>,
        on_double_click: Rc<RefCell<Option<Box<dyn Fn(PathBuf) -> Message>>>>,
        show_hidden: bool,
        show_extensions: bool,
    ) -> Self {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();

        Self {
            path,
            name,
            files: OnceCell::default(),
            folders: OnceCell::default(),
            empty_files,
            empty_folders,
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

        self.folders
            .get_or_init(|| state.folders.get_or_init(|| self.init_folders()).clone())
            .iter()
            .cloned()
            .map(Into::into)
            .chain(
                self.files
                    .get_or_init(|| state.files.get_or_init(|| self.init_files()).clone())
                    .iter()
                    .cloned()
                    .map(Into::into),
            )
            .collect()
    }

    fn get_children(&self) -> Vec<Element<'_, Message, Theme, Renderer>> {
        self.folders
            .get()
            .unwrap_or(&self.empty_folders)
            .iter()
            .cloned()
            .map(Into::into)
            .chain(
                self.files
                    .get()
                    .unwrap_or(&self.empty_files)
                    .iter()
                    .cloned()
                    .map(Into::into),
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

    fn init_folders(&self) -> Rc<[Self]> {
        let Ok(folders) = std::fs::read_dir(&self.path) else {
            return [].into();
        };

        let mut folders: Box<_> = folders
            .filter_map(Result::ok)
            .filter(|file| file.file_type().is_ok_and(|t| t.is_dir()))
            .map(|file| {
                let mut name = file.file_name();
                name.make_ascii_lowercase();

                (file, name)
            })
            .filter(|(_, name)| !self.show_hidden && !name.as_encoded_bytes().starts_with(b"."))
            .collect();
        folders.sort_by(|(_, aname), (_, bname)| aname.cmp(bname));
        folders
            .iter()
            .map(|(entry, _)| {
                let path = entry.path();
                Self::new_inner(
                    path,
                    self.empty_files.clone(),
                    self.empty_folders.clone(),
                    self.on_single_click.clone(),
                    self.on_double_click.clone(),
                    self.show_hidden,
                    self.show_extensions,
                )
            })
            .collect()
    }
}

impl<Message> Widget<Message, Theme, Renderer> for Folder<Message>
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
            return Node::new(Size::new(
                limits.max().width,
                *state
                    .line_height
                    .get_or_init(|| (renderer.default_size().0 * 1.3).ceil()),
            ));
        }

        self.diff(tree);

        let state = tree.state.downcast_ref::<State<Message>>();

        let layout = layout::flex::resolve(
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
            &self.get_children(),
            &mut tree.children,
        );

        layout
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

        if let Some(pos) = cursor.position() {
            if event == Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                && layout.bounds().contains(pos)
                && &cursor.position_in(layout.bounds()).unwrap().y
                    <= state.line_height.get().unwrap()
            {
                state.open ^= true;
                shell.invalidate_layout();
                return Status::Captured;
            }
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
        let state = tree.state.downcast_ref::<State<Message>>();
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
                state.line_height.get().unwrap() * 0.5 - 1.0,
                state.line_height.get().unwrap() * 1.5 - 1.0,
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

impl<Message> From<Folder<Message>> for Element<'_, Message, Theme, Renderer>
where
    Message: Clone + 'static,
{
    fn from(folder: Folder<Message>) -> Self {
        Self::new(folder)
    }
}
