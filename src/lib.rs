mod file;
pub(crate) use file::File;

mod folder;

pub(crate) static SPACING: f32 = 1.0;

pub type FileTree<'a, Message> = folder::Folder<'a, Message>;
