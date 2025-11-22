use native_windows_gui::Font;
use std::sync::LazyLock;

pub static INFO_LABEL_FONT: LazyLock<Font> = LazyLock::new(|| {
    let mut font = Font::default();
    Font::builder()
        .family("Consolas")
        .size(28)
        .weight(700)
        .build(&mut font)
        .expect("Failed to build font");
    font
});

pub static SMALL_MONO_FONT: LazyLock<Font> = LazyLock::new(|| mono_font(15));

pub static BIG_MONO_FONT: LazyLock<Font> = LazyLock::new(|| mono_font(18));

pub(crate) fn display_font(size: u32) -> Font {
    let mut font = Font::default();
    Font::builder()
        .family("Segoe UI")
        .size(size)
        .build(&mut font)
        .expect("Failed to build font");
    font
}

fn mono_font(size: u32) -> Font {
    let mut font = Font::default();
    Font::builder()
        .family("Consolas")
        .size(size)
        .build(&mut font)
        .expect("Failed to build font");
    font
}
