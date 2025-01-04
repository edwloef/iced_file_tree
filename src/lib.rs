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

mod dir;
mod file;
mod file_tree;
pub use file_tree::{file_tree, FileTree};

const LINE_HEIGHT: f32 = 21.0;
