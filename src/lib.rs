mod file;
pub(crate) use file::File;

mod folder;

pub(crate) static SPACING: f32 = 1.0;

#[doc(inline)]
pub use folder::Folder as FileTree;
