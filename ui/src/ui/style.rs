use native_windows_gui as nwg;
use native_windows_gui::stretch::geometry::Rect;
use nwg::stretch::style::Dimension::Points as PT;
use std::sync::LazyLock;
use native_windows_gui::stretch::style::Dimension;

pub static PADDING: Rect<Dimension> = Rect {
    // start: PT(4.0),
    // end: PT(4.0),
    // top: PT(4.0),
    // bottom: PT(4.0),
    start: PT(0.0),
    end: PT(0.0),
    top: PT(0.0),
    bottom: PT(0.0),
};

pub static TAB_PADDING: Rect<Dimension> = Rect {
    // start: PT(0.0),
    // end: PT(8.0),
    // top: PT(0.0),
    // bottom: PT(4.0),
    start: PT(0.0),
    end: PT(0.0),
    top: PT(0.0),
    bottom: PT(0.0),
};

pub static MARGIN: Rect<Dimension> = Rect {
//     start: PT(4.0),
//     end: PT(4.0),
//     top: PT(4.0),
//     bottom: PT(4.0),
    start: PT(0.0),
    end: PT(0.0),
    top: PT(0.0),
    bottom: PT(0.0),
};

pub static MARGIN_2: Rect<Dimension> = Rect {
    // start: PT(12.0),
    // end: PT(12.0),
    // top: PT(4.0),
    // bottom: PT(4.0),
    start: PT(0.0),
    end: PT(0.0),
    top: PT(0.0),
    bottom: PT(0.0),
};

pub static TAB_MARGIN: Rect<Dimension> = Rect {
    // start: PT(4.0),
    // end: PT(4.0),
    // top: PT(4.0),
    // bottom: PT(18.0),
    start: PT(0.0),
    end: PT(0.0),
    top: PT(0.0),
    bottom: PT(0.0),
};

pub static INFO_LABEL_FONT: LazyLock<nwg::Font> = LazyLock::new(|| {
    let mut font = nwg::Font::default();
    nwg::Font::builder()
        .family("Consolas")
        .size(28)
        .weight(700)
        .build(&mut font)
        .expect("Failed to build font");
    font
});

pub static SMALL_MONO_FONT: LazyLock<nwg::Font> = LazyLock::new(|| mono_font(15));

pub static BIG_MONO_FONT: LazyLock<nwg::Font> = LazyLock::new(|| mono_font(18));

pub(crate) fn display_font(size: u32) -> nwg::Font {
    let mut font = nwg::Font::default();
    nwg::Font::builder()
        .family("Segoe UI")
        .size(size)
        .build(&mut font)
        .expect("Failed to build font");
    font
}

fn mono_font(size: u32) -> nwg::Font {
    let mut font = nwg::Font::default();
    nwg::Font::builder()
        .family("Consolas")
        .size(size)
        .build(&mut font)
        .expect("Failed to build font");
    font
}
