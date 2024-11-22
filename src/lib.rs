//! A lightweight file tree widget for the [iced](https://github.com/iced-rs/iced/tree/master) toolkit.
//!
//! # Example
//! ```no_run
//! use iced::widget::scrollable;
//! use iced_file_tree::file_tree;
//!
//! enum Message {
//!     FileTreeMessage(PathBuf),
//!     // ...
//! }
//!
//! fn view(state: &State) -> Element<'_, Message> {
//!     let path: PathBuf = /* */
//!
//!     scrollable(
//!         file_tree(path)
//!             .unwrap()
//!             .on_double_click(Message::FileTreeMessage),
//!     )
//!     .into()
//! }
//! ```

mod errentry;
mod file;
mod folder;

#[doc(inline)]
pub use folder::Folder as FileTree;

use std::path::PathBuf;

/// Creates a new [`FileTree`] with the root at the given path.
#[must_use]
pub fn file_tree<'a, Message>(path: PathBuf) -> Option<FileTree<'a, Message>>
where
    Message: Clone + 'a,
{
    FileTree::<'a, Message>::new(path)
}
