use crate::res::res_ids::{
    IDS_ACTION, IDS_KEY, IDS_MODIFIERS, IDS_RULE, IDS_SCAN_CODE, IDS_STATUS, IDS_TIME,
    IDS_TRANSITION, IDS_VIRTUAL_KEY,
};
use crate::rs;
use crate::ui::RESOURCES;
use crate::utils::hwnd;
use keympostor::ife;
use keympostor::keyboard::event::KeyEvent;
use keympostor::keyboard::modifiers::{
    KM_LALT, KM_LCTRL, KM_LSHIFT, KM_LWIN, KM_RALT, KM_RCTRL, KM_RSHIFT, KM_RWIN,
};
use keympostor::keyboard::trigger::KeyTrigger;
use native_windows_gui::{
    ControlHandle, InsertListViewColumn, ListView, ListViewColumnFlags, ListViewExFlags,
    ListViewStyle, NwgError, Tab,
};
use windows::Win32::Foundation::WPARAM;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::SendMessageW;

const MAX_LOG_ITEMS: usize = 256;

#[derive(Default)]
pub(crate) struct LogView {
    list: ListView,
}

impl LogView {
    pub(crate) fn build(&mut self, parent: &Tab) -> Result<(), NwgError> {
        ListView::builder()
            .parent(parent)
            .list_style(ListViewStyle::Detailed)
            .ex_flags(ListViewExFlags::FULL_ROW_SELECT)
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
            width: Some(0),
            text: Some(rs!(IDS_MODIFIERS).into()),
        });

        self.list.insert_column(InsertListViewColumn {
            index: Some(3),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(0),
            text: Some(rs!(IDS_KEY).into()),
        });

        self.list.insert_column(InsertListViewColumn {
            index: Some(4),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(0),
            text: Some(rs!(IDS_TRANSITION).into()),
        });

        self.list.insert_column(InsertListViewColumn {
            index: Some(5),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(0),
            text: Some(rs!(IDS_VIRTUAL_KEY).into()),
        });

        self.list.insert_column(InsertListViewColumn {
            index: Some(6),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(0),
            text: Some(rs!(IDS_SCAN_CODE).into()),
        });

        self.list.insert_column(InsertListViewColumn {
            index: Some(7),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(0),
            text: Some(rs!(IDS_TIME).into()),
        });

        self.list.insert_column(InsertListViewColumn {
            index: Some(8),
            fmt: Some(ListViewColumnFlags::RIGHT),
            width: Some(0),
            text: Some(rs!(IDS_STATUS).into()),
        });

        Ok(())
    }

    pub(crate) fn view(&self) -> impl Into<ControlHandle> {
        &self.list
    }

    pub(crate) fn append(&self, event: &KeyEvent) {
        self.list.set_redraw(false);

        while self.list.len() > MAX_LOG_ITEMS {
            self.list.remove_item(0);
        }

        self.list.insert_items_row(
            None,
            &[
                KeyTrigger::from(event).to_string(),
                event.rule.map(|r| r.to_string()).unwrap_or("".to_string()),
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
                event.action.key.virtual_key().to_string(),
                event.action.key.scan_code().to_string(),
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
        self.scroll_to_end();
    }

    pub(crate) fn clear(&self) {
        self.list.clear()
    }

    fn scroll_to_end(&self) {
        let len = self.list.len();
        if len > 0 {
            let hwnd = hwnd(self.list.handle).unwrap();
            unsafe {
                SendMessageW(hwnd, LVM_ENSUREVISIBLE, Some(WPARAM(len - 1)), None);
            }
        }
    }

    // pub(crate) fn handle_raw_event(&self, msg: u32, l_param: isize) {
    //     if msg == WM_NOTIFY {
    //         unsafe {
    //             debug!("WM_NOTIFY");
    //             let nmhdr = l_param as *mut NMHDR;
    //             if (*nmhdr).code == NM_CUSTOMDRAW {
    //                 let cd = &mut *(l_param as *mut NMLVCUSTOMDRAW);
    //                 match cd.nmcd.dwDrawStage {
    //                     CDDS_ITEMPREPAINT => {
    //                         let index = cd.nmcd.dwItemSpec as usize;
    //
    //                         // например, красим чётные строки
    //                         if index % 2 == 0 {
    //                             cd.clrTextBk = COLORREF(RGB(240, 240, 255)); // светло-синий фон
    //                         } else {
    //                             cd.clrTextBk = COLORREF(RGB(255, 255, 255)); // белый фон
    //                         }
    //
    //                         // Some(CDRF_NEWFONT as isize)
    //                     }
    //                     _ => {}
    //                 }
    //             }
    //         }
    //     }
}
