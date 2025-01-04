use crate::dir::Dir;
use iced::{
    advanced::{
        layout::{Limits, Node},
        renderer::Style,
        widget::{tree, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    event::Status,
    mouse::Cursor,
    Element, Event, Length, Rectangle, Renderer, Size, Theme,
};
use std::{
    fmt::{Debug, Formatter},
    path::PathBuf,
    rc::Rc,
};

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
pub struct FileTree<Message>(Dir<Message>);

impl<Message> Debug for FileTree<Message> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Dir")
            .field("path", &self.0.path)
            .field("show_hidden", &self.0.show_hidden)
            .field("show_extensions", &self.0.show_hidden)
            .finish()
    }
}

/// Creates a new [`FileTree`] with the root at the given path.
#[must_use]
pub fn file_tree<Message>(path: PathBuf) -> Option<FileTree<Message>>
where
    Message: Clone + 'static,
{
    FileTree::<Message>::new(path)
}

impl<Message> FileTree<Message>
where
    Message: Clone + 'static,
{
    /// Creates a new [`FileTree`] with the root at the given path.
    #[must_use]
    pub fn new(path: PathBuf) -> Option<Self> {
        if std::fs::read_dir(&path).is_err() {
            return None;
        }

        Some(Self(Dir::new_inner(
            path,
            Rc::default(),
            Rc::default(),
            false,
            true,
        )))
    }

    /// Sets the message that will be produced when the user single-clicks on a file within the [`FileTree`].
    #[must_use]
    pub fn on_single_click(self, on_single_click: impl Fn(PathBuf) -> Message + 'static) -> Self {
        self.0
            .on_single_click
            .borrow_mut()
            .replace(Box::new(on_single_click));
        self
    }

    /// Sets the message that will be produced when the user double-clicks on a file within the [`FileTree`].
    #[must_use]
    pub fn on_double_click(self, on_double_click: impl Fn(PathBuf) -> Message + 'static) -> Self {
        self.0
            .on_double_click
            .borrow_mut()
            .replace(Box::new(on_double_click));
        self
    }

    /// Enables or disables showing hidden files (disabled by default).
    #[must_use]
    pub fn hidden_files(mut self, show_hidden: bool) -> Self {
        self.0.show_hidden = show_hidden;
        self
    }

    #[must_use]
    /// Enables or disables showing file extensions (enabled by default).
    pub fn file_extensions(mut self, show_extensions: bool) -> Self {
        self.0.show_extensions = show_extensions;
        self
    }
}

impl<Message> Widget<Message, Theme, Renderer> for FileTree<Message>
where
    Message: Clone + 'static,
{
    fn children(&self) -> Vec<Tree> {
        self.0.children()
    }

    fn size(&self) -> Size<Length> {
        self.0.size()
    }

    fn tag(&self) -> tree::Tag {
        self.0.tag()
    }

    fn state(&self) -> tree::State {
        self.0.state()
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        self.0.layout(tree, renderer, limits)
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
        self.0.on_event(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
        )
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

        renderer.with_layer(bounds, |renderer| {
            self.0
                .draw(tree, renderer, theme, style, layout, cursor, &bounds);
        });
    }
}

impl<Message> From<FileTree<Message>> for Element<'_, Message, Theme, Renderer>
where
    Message: Clone + 'static,
{
    fn from(dir: FileTree<Message>) -> Self {
        Self::new(dir)
    }
}
