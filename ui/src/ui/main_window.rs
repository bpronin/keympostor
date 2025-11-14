use super::*;
use crate::res::RESOURCES;
use crate::res::res_ids::{IDI_ICON_APP, IDS_APP_TITLE, IDS_LAYOUT, IDS_LOG};
use crate::ui::style::INFO_LABEL_FONT;
use crate::{r_icon, rs};
use native_windows_gui as nwg;
use native_windows_gui::NwgError;
use native_windows_gui::stretch::geometry::Rect;
use native_windows_gui::stretch::style::Dimension;
use nwg::stretch::style::Dimension::Points as PT;

#[derive(Default)]
pub(crate) struct MainWindow {
    pub(crate) window: nwg::Window,
    layout: nwg::FlexboxLayout,
    tab_log_layout: nwg::FlexboxLayout,
    tab_layouts_layout: nwg::FlexboxLayout,
    tab_log: nwg::Tab,
    tab_layouts: nwg::Tab,
    pub(crate) main_menu: MainMenu,
    pub(crate) tab_container: nwg::TabsContainer,
    pub(crate) layout_view: LayoutView,
    pub(crate) log_view: LogView,
    pub(crate) key_event_label: nwg::Label,
    pub(crate) test_editor: TypeTestEditor,
}

impl MainWindow {
    pub(crate) fn build(&mut self) -> Result<(), NwgError> {
        nwg::Window::builder()
            .size((700, 300))
            .icon(Some(r_icon!(IDI_ICON_APP)))
            .flags(nwg::WindowFlags::MAIN_WINDOW)
            .title(rs!(IDS_APP_TITLE))
            .build(&mut self.window)?;

        nwg::Label::builder()
            .parent(&self.window)
            .text("*")
            .font(Some(&INFO_LABEL_FONT))
            .build(&mut self.key_event_label)?;

        self.test_editor.build(&mut self.window)?;

        /* Tabs */

        nwg::TabsContainer::builder()
            .parent(&self.window)
            .build(&mut self.tab_container)?;

        nwg::Tab::builder()
            .text(rs!(IDS_LOG))
            .parent(&self.tab_container)
            .build(&mut self.tab_log)?;

        nwg::Tab::builder()
            .text(rs!(IDS_LAYOUT))
            .parent(&self.tab_container)
            .build(&mut self.tab_layouts)?;

        self.main_menu.build(&mut self.window)?;
        self.log_view.build(&mut self.tab_log)?;
        self.layout_view.build(&mut self.tab_layouts)?;

        self.layout()
    }

    fn layout(&self) -> Result<(), nwg::NwgError> {
        use nwg::stretch::{geometry::Size, style::FlexDirection};

        /* Log tab layout */

        nwg::FlexboxLayout::builder()
            .parent(&self.tab_container)
            // .padding(TAB_PADDING)
            .child(self.log_view.view())
            .child_margin(Rect {
                start: PT(4.0),
                end: PT(16.0),
                top: PT(6.0),
                bottom: PT(40.0),
            })
            .build(&self.tab_log_layout)?;

        /* Layout tab layout */

        nwg::FlexboxLayout::builder()
            .parent(&self.tab_container)
            // .padding(TAB_PADDING)
            .child(self.layout_view.view())
            .child_margin(Rect {
                start: PT(4.0),
                end: PT(16.0),
                top: PT(6.0),
                bottom: PT(40.0),
            })
            .build(&self.tab_layouts_layout)?;

        /* Main window layout */

        nwg::FlexboxLayout::builder()
            .parent(&self.window)
            .flex_direction(FlexDirection::Column)
            // .padding(PADDING)
            /* Tabs */
            .child(&self.tab_container)
            // .child_margin(MARGIN)
            .child_flex_grow(1.0)
            /* Test label */
            .child(&self.key_event_label)
            // .child_margin(MARGIN_2)
            .child_size(Size {
                width: Dimension::Auto,
                height: Dimension::Points(24.0),
            })
            /* Test editor */
            .child(self.test_editor.editor())
            // .child_margin(MARGIN_2)
            .child_size(Size {
                width: Dimension::Auto,
                height: Dimension::Points(40.0),
            })
            .build(&self.layout)
    }
}
