use crate::utils::raw_hwnd;
use keympostor::ife;
use keympostor::keyboard::event::KeyEvent;
use keympostor::keyboard::modifiers::{
    KM_LALT, KM_LCTRL, KM_LSHIFT, KM_LWIN, KM_RALT, KM_RCTRL, KM_RSHIFT, KM_RWIN,
};
use keympostor::keyboard::trigger::KeyTrigger;
use native_windows_gui as nwg;
use native_windows_gui::{ListViewExFlags, ListViewStyle};
use windows::Win32::Foundation::WPARAM;
use windows::Win32::UI::Controls::*;
use windows::Win32::UI::WindowsAndMessaging::SendMessageW;
const MAX_LOG_LINES: usize = 256;

#[derive(Default)]
pub(crate) struct LogView {
    list: nwg::ListView,
}

impl LogView {
    pub(crate) fn build_ui(&mut self, parent: &nwg::Tab) -> Result<(), nwg::NwgError> {
        nwg::ListView::builder()
            .parent(parent)
            .list_style(ListViewStyle::Detailed)
            .ex_flags(ListViewExFlags::FULL_ROW_SELECT)
            .build(&mut self.list)?;

        // self.view.set_headers_enabled(true);

        self.list.insert_column(nwg::InsertListViewColumn {
            index: Some(0),
            fmt: Some(nwg::ListViewColumnFlags::RIGHT),
            width: Some(500),
            text: Some("Action".into()),
        });

        self.list.insert_column(nwg::InsertListViewColumn {
            index: Some(1),
            fmt: Some(nwg::ListViewColumnFlags::LEFT),
            width: Some(0),
            text: Some("Modifiers".into()),
        });

        self.list.insert_column(nwg::InsertListViewColumn {
            index: Some(2),
            fmt: Some(nwg::ListViewColumnFlags::LEFT),
            width: Some(0),
            text: Some("Key".into()),
        });

        self.list.insert_column(nwg::InsertListViewColumn {
            index: Some(3),
            fmt: Some(nwg::ListViewColumnFlags::LEFT),
            width: Some(0),
            text: Some("Transition".into()),
        });

        self.list.insert_column(nwg::InsertListViewColumn {
            index: Some(4),
            fmt: Some(nwg::ListViewColumnFlags::LEFT),
            width: Some(0),
            text: Some("VK".into()),
        });

        self.list.insert_column(nwg::InsertListViewColumn {
            index: Some(5),
            fmt: Some(nwg::ListViewColumnFlags::LEFT),
            width: Some(0),
            text: Some("SC".into()),
        });

        self.list.insert_column(nwg::InsertListViewColumn {
            index: Some(6),
            fmt: Some(nwg::ListViewColumnFlags::LEFT),
            width: Some(0),
            text: Some("Time".into()),
        });

        self.list.insert_column(nwg::InsertListViewColumn {
            index: Some(7),
            fmt: Some(nwg::ListViewColumnFlags::RIGHT),
            width: Some(0),
            text: Some("Status".into()),
        });

        Ok(())
    }

    pub(crate) fn view(&self) -> impl Into<nwg::ControlHandle> {
        &self.list
    }

    pub(crate) fn append(&self, event: &KeyEvent) {
        self.list.insert_items_row(
            None,
            &[
                KeyTrigger::from(event).to_string(),
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
        self.scroll_to_end();
    }

    pub(crate) fn clear(&self) {
        self.list.clear()
    }

    fn scroll_to_end(&self) {
        let len = self.list.len();
        if len > 0 {
            let hwnd = raw_hwnd(self.list.handle).unwrap();
            unsafe {
                SendMessageW(hwnd, LVM_ENSUREVISIBLE, Some(WPARAM(len - 1)), None);
            }
        }
    }

    // }

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
