use crate::res::RESOURCES;
use crate::res::res_ids::{
    IDS_ACTION, IDS_KEY, IDS_MODIFIERS, IDS_RULE, IDS_SCAN_CODE, IDS_STATUS, IDS_TIME,
    IDS_TRANSITION, IDS_VIRTUAL_KEY,
};
use crate::rs;
use crate::settings::MainWindowSettings;
use crate::ui::utils::get_list_view_column_width;
use crate::ui::utils::scroll_list_view_to_end;
use keympostor::event::KeyEvent;
use keympostor::ife;
use keympostor::modifiers::{
    KM_LALT, KM_LCTRL, KM_LSHIFT, KM_LWIN, KM_RALT, KM_RCTRL, KM_RSHIFT, KM_RWIN,
};
use log::error;
use native_windows_gui::{
    unbind_raw_event_handler, ControlHandle, InsertListViewColumn, ListView, ListViewColumnFlags, ListViewStyle,
    NwgError, RawEventHandler, Tab,
};
use std::cell::RefCell;
use std::collections::HashMap;

const MAX_LOG_ITEMS: usize = 256;

#[derive(Default)]
pub(crate) struct LogView {
    list: ListView,
    raw_event_handler: RefCell<Option<RawEventHandler>>,
}

impl Drop for LogView {
    fn drop(&mut self) {
        if let Some(handler) = self.raw_event_handler.borrow().as_ref() {
            unbind_raw_event_handler(handler)
                .unwrap_or_else(|e| error!("Failed to unbind raw event handler: {}", e));
        }
    }
}

impl LogView {
    pub(crate) fn build(&mut self, parent: &Tab) -> Result<(), NwgError> {
        ListView::builder()
            .parent(parent)
            .list_style(ListViewStyle::Detailed)
            // .ex_flags(ListViewExFlags::FULL_ROW_SELECT)
            .build(&mut self.list)?;

        self.list.set_headers_enabled(true);

        self.list.insert_column(InsertListViewColumn {
            index: Some(0),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(300),
            text: Some(rs!(IDS_ACTION).into()),
        });

        self.list.insert_column(InsertListViewColumn {
            index: Some(1),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(500),
            text: Some(rs!(IDS_RULE).into()),
        });

        self.list.insert_column(InsertListViewColumn {
            index: Some(2),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(50),
            text: Some(rs!(IDS_MODIFIERS).into()),
        });

        self.list.insert_column(InsertListViewColumn {
            index: Some(3),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(50),
            text: Some(rs!(IDS_KEY).into()),
        });

        self.list.insert_column(InsertListViewColumn {
            index: Some(4),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(50),
            text: Some(rs!(IDS_TRANSITION).into()),
        });

        self.list.insert_column(InsertListViewColumn {
            index: Some(5),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(50),
            text: Some(rs!(IDS_VIRTUAL_KEY).into()),
        });

        self.list.insert_column(InsertListViewColumn {
            index: Some(6),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(50),
            text: Some(rs!(IDS_SCAN_CODE).into()),
        });

        self.list.insert_column(InsertListViewColumn {
            index: Some(7),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(50),
            text: Some(rs!(IDS_TIME).into()),
        });

        self.list.insert_column(InsertListViewColumn {
            index: Some(8),
            fmt: Some(ListViewColumnFlags::RIGHT),
            width: Some(50),
            text: Some(rs!(IDS_STATUS).into()),
        });

        // self.raw_event_handler.replace(Some(
        //     bind_raw_event_handler(
        //         &self.list.handle,
        //         0x10000,
        //         move |_hwnd, msg, _w_param, l_param| {
        //             Self::handle_raw_event(msg, l_param)
        //         },
        //     )
        //     .expect("Failed to bind raw event handler"),
        // ));

        Ok(())
    }

    pub(crate) fn view(&self) -> impl Into<ControlHandle> {
        &self.list
    }

    pub(crate) fn apply_settings(&self, settings: &MainWindowSettings) {
        if let Some(columns) = &settings.log_view.columns {
            for i in 0..self.list.column_len() {
                if let Some(w) = columns.get(&i) {
                    self.list.set_column_width(i, *w);
                }
            }
        }
    }

    pub(crate) fn update_settings(&self, settings: &mut MainWindowSettings) {
        let mut map = HashMap::new();
        for i in 0..self.list.column_len() {
            map.insert(i, get_list_view_column_width(&self.list, i));
        }
        settings.log_view.columns = Some(map);
    }

    pub(crate) fn append(&self, event: &KeyEvent) {
        self.list.set_redraw(false);

        while self.list.len() > MAX_LOG_ITEMS {
            self.list.remove_item(0);
        }

        self.list.insert_items_row(
            None,
            &[
                event.as_trigger().to_string(),
                event
                    .rule
                    .as_ref()
                    .map(|r| r.to_string())
                    .unwrap_or(String::from("")),
                format!(
                    "{:1} {:1} {:1} {:1} {:1} {:1} {:1} {:1}",
                    ife!(event.modifiers.contains(KM_LSHIFT), "S", "."),
                    ife!(event.modifiers.contains(KM_LCTRL), "C", "."),
                    ife!(event.modifiers.contains(KM_LWIN), "W", "."),
                    ife!(event.modifiers.contains(KM_LALT), "A", "."),
                    ife!(event.modifiers.contains(KM_RALT), "A", "."),
                    ife!(event.modifiers.contains(KM_RWIN), "W", "."),
                    ife!(event.modifiers.contains(KM_RCTRL), "C", "."),
                    ife!(event.modifiers.contains(KM_RSHIFT), "S", "."),
                ),
                event.action.key.to_string(),
                event.action.transition.to_string(),
                event.action.key.vk.to_string(),
                event.action.key.sc.to_string(),
                event.time.to_string(),
                format!(
                    "{:1}{:1}{:1}",
                    if event.rule.is_some() { "!" } else { "" },
                    if event.is_injected { ">" } else { "" },
                    if event.is_private { "<" } else { "" },
                ),
            ],
        );

        self.list.set_redraw(true);

        scroll_list_view_to_end(&self.list);
    }

    pub(crate) fn clear(&self) {
        self.list.clear()
    }

    // pub(crate) fn handle_raw_event(msg: u32, l_param: isize) -> Option<isize>{
    //     if msg == WM_NOTIFY {
    //         unsafe {
    //             let nmhdr = l_param as *mut NMHDR;
    //             if (*nmhdr).code == NM_CUSTOMDRAW {
    //                 let cd = &mut *(l_param as *mut NMLVCUSTOMDRAW);
    //                 let result = match cd.nmcd.dwDrawStage {
    //                     CDDS_PREPAINT => {
    //                         CDRF_NOTIFYITEMDRAW
    //                     }
    //                     CDDS_ITEMPREPAINT => {
    //                         let index = cd.nmcd.dwItemSpec;
    //                         if index % 2 == 0 {
    //                             cd.clrTextBk = COLORREF(RGB(255, 0, 0));
    //                             CDRF_NEWFONT
    //                         } else {
    //                             CDRF_DODEFAULT
    //                         }
    //                     }
    //                     CDDS_SUBITEM | CDDS_ITEMPREPAINT => {
    //                         CDRF_NEWFONT
    //                     }
    //                     _ => CDRF_DODEFAULT
    //                 };
    //                 debug!("Custom draw result: {}", result);
    //                 return Some(result as isize);
    //             }
    //         }
    //     }
    //     None
    // }
}
