mod err;

mod file;
pub(crate) use file::File;

mod folder;

#[doc(inline)]
pub use folder::Folder as FileTree;
