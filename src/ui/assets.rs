use std::sync::LazyLock;

use iced::widget::svg::Handle;

pub static ARROW_LEFT: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/arrow-left.svg")));
pub static ARROW_RIGHT: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/arrow-right.svg")));
pub static BOOKMARK: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/bookmark.svg")));
pub static BOOKMARK_FILLED: LazyLock<Handle> = LazyLock::new(|| {
    Handle::from_memory(include_bytes!("../../assets/images/bookmark-filled.svg"))
});
pub static CHECK: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/check.svg")));
pub static CHECKED_CIRCLE: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/check-circle.svg")));
pub static CROSS: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/cross.svg")));
pub static CROSS_CIRCLE: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/cross-circle.svg")));
pub static DOWN_TICK: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/down-tick.svg")));
pub static DOWNLOAD_ARROW: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/download-arrow.svg")));
pub static EXCLAMATION_CIRCLE: LazyLock<Handle> = LazyLock::new(|| {
    Handle::from_memory(include_bytes!("../../assets/images/exclamation-circle.svg"))
});
pub static FILE: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/file.svg")));
pub static FILE_COPY: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/file-copy.svg")));
pub static GLOBE: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/globe.svg")));
pub static HOME: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/home.svg")));
pub static MINUS_CIRCLE: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/minus-circle.svg")));
pub static QUESTION_CIRCLE: LazyLock<Handle> = LazyLock::new(|| {
    Handle::from_memory(include_bytes!("../../assets/images/question-circle.svg"))
});
pub static REFRESH: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/refresh.svg")));
pub static SAVE_FILE: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/save-file.svg")));
pub static SETTINGS: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/settings.svg")));
pub static SQUARE: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/square.svg")));
pub static SQUARE_FILLED: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/square-filled.svg")));
pub static SWITCH: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/switch.svg")));
pub static TRASH: LazyLock<Handle> =
    LazyLock::new(|| Handle::from_memory(include_bytes!("../../assets/images/trash.svg")));

/// Just the bytes of the app icon png file.
pub static APP_ICON: &[u8] = include_bytes!("../../assets/logo/logo-1024.png");
