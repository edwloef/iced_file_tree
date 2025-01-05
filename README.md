# Iced File Tree

[![Crates.io](https://img.shields.io/crates/v/iced_file_tree.svg)](https://crates.io/crates/iced_file_tree)
[![Documentation](https://docs.rs/iced_file_tree/badge.svg)](https://docs.rs/iced_file_tree)
[![Iced](https://img.shields.io/badge/0.13-blue.svg?logo=iced)](https://github.com/iced-rs/iced/tree/master)
[![License](https://img.shields.io/crates/l/iced_file_tree.svg)](https://github.com/edwloef/iced-file-tree/blob/master/LICENSE)

A lightweight file tree widget for the [iced](https://github.com/iced-rs/iced/tree/master) toolkit.

## Usage

Include `iced_file_tree` as a dependency in your `Cargo.toml`:

```toml
[dependencies]
iced = "0.13.1"
iced_file_tree = "0.2.0"
```

### Example
```rs
use iced::widget::scrollable;
use iced_file_tree::file_tree;

enum Message {
    FileTreeMessage(PathBuf),
    // ...
}

fn view(state: &State) -> Element<'_, Message> {
    let path: PathBuf = // ...

    scrollable(
        file_tree(path)
            .unwrap()
            .on_double_click(Message::FileTreeMessage),
    )
    .into()
}
```

The `FileTree` widget is recommended to be put in an iced [`Scrollable`](https://docs.rs/iced/latest/iced/widget/scrollable/).
