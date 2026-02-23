use crate::rs;
use crate::settings::MainWindowSettings;
use crate::ui::res::RESOURCES;
use crate::ui::res_ids::{
    IDS_ACTION, IDS_KEY, IDS_MODIFIERS, IDS_RULE, IDS_SCAN_CODE, IDS_STATUS, IDS_TIME,
    IDS_TRANSITION, IDS_VIRTUAL_KEY,
};
use crate::ui::utils::get_list_view_column_width;
use crate::ui::utils::{scroll_list_view_to_end, set_list_view_item_data};
use keympostor::notify::KeyEventNotification;
use keympostor::utils::if_else;
use native_windows_gui::{
    bind_raw_event_handler, ControlHandle, InsertListViewColumn, ListView, ListViewColumnFlags,
    ListViewExFlags, ListViewStyle, NwgError, Tab,
};
use std::collections::HashMap;
use windows::Win32::Foundation::COLORREF;
use windows::Win32::UI::Controls::{
    CDDS_ITEMPREPAINT, CDDS_PREPAINT, CDRF_DODEFAULT, CDRF_NEWFONT, CDRF_NOTIFYITEMDRAW,
    NMHDR, NMLVCUSTOMDRAW, NM_CUSTOMDRAW,
};
use windows::Win32::UI::WindowsAndMessaging::WM_NOTIFY;

const MAX_LOG_ITEMS: usize = 256;

#[derive(Default)]
pub(crate) struct LogView {
    list_view: ListView,
}

impl LogView {
    pub(crate) fn build(&mut self, parent: &Tab) -> Result<(), NwgError> {
        ListView::builder()
            .parent(parent)
            .list_style(ListViewStyle::Detailed)
            .ex_flags(ListViewExFlags::GRID)
            .build(&mut self.list_view)?;

        self.list_view.set_headers_enabled(true);

        self.list_view.insert_column(InsertListViewColumn {
            index: Some(0),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(300),
            text: Some(rs!(IDS_ACTION).into()),
        });

        self.list_view.insert_column(InsertListViewColumn {
            index: Some(1),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(500),
            text: Some(rs!(IDS_RULE).into()),
        });

        self.list_view.insert_column(InsertListViewColumn {
            index: Some(2),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(50),
            text: Some(rs!(IDS_MODIFIERS).into()),
        });

        self.list_view.insert_column(InsertListViewColumn {
            index: Some(3),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(50),
            text: Some(rs!(IDS_KEY).into()),
        });

        self.list_view.insert_column(InsertListViewColumn {
            index: Some(4),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(50),
            text: Some(rs!(IDS_TRANSITION).into()),
        });

        self.list_view.insert_column(InsertListViewColumn {
            index: Some(5),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(50),
            text: Some(rs!(IDS_VIRTUAL_KEY).into()),
        });

        self.list_view.insert_column(InsertListViewColumn {
            index: Some(6),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(50),
            text: Some(rs!(IDS_SCAN_CODE).into()),
        });

        self.list_view.insert_column(InsertListViewColumn {
            index: Some(7),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(50),
            text: Some(rs!(IDS_TIME).into()),
        });

        self.list_view.insert_column(InsertListViewColumn {
            index: Some(8),
            fmt: Some(ListViewColumnFlags::RIGHT),
            width: Some(50),
            text: Some(rs!(IDS_STATUS).into()),
        });

        bind_raw_event_handler(
            &parent.handle,
            0x10001,
            move |_hwnd, msg, _w_param, l_param| Self::handle_custom_draw(msg, l_param),
        )?;

        Ok(())
    }

    pub(crate) fn view(&self) -> impl Into<ControlHandle> {
        &self.list_view
    }

    pub(crate) fn apply_settings(&self, settings: &MainWindowSettings) {
        if let Some(columns) = &settings.log_view.columns {
            for i in 0..self.list_view.column_len() {
                if let Some(w) = columns.get(&i) {
                    self.list_view.set_column_width(i, *w);
                }
            }
        }
    }

    pub(crate) fn update_settings(&self, settings: &mut MainWindowSettings) {
        let mut map = HashMap::new();
        for i in 0..self.list_view.column_len() {
            map.insert(i, get_list_view_column_width(&self.list_view, i));
        }
        settings.log_view.columns = Some(map);
    }

    pub(crate) fn append(&self, notification: &KeyEventNotification) {
        self.list_view.set_redraw(false);

        while self.list_view.len() > MAX_LOG_ITEMS {
            self.list_view.remove_item(0);
        }

        let event = &notification.event;
        let trigger = &event.trigger;
        let rule = notification.rule.as_ref();

        self.list_view.insert_items_row(
            None,
            &[
                trigger.to_string(),
                rule.map(|r| r.to_string()).unwrap_or("".to_string()),
                trigger.modifiers.to_string(),
                trigger.action.key.to_string(),
                trigger.action.transition.to_string(),
                format!("0x{:02X}", trigger.action.key.vk()),
                format!("0x{:04X}", trigger.action.key.sc_ext()),
                event.time.to_string(),
                format!(
                    "{:1}{:1}{:1}",
                    if_else(rule.is_some(), "R", "-"),
                    if_else(event.is_injected, "I", "-"),
                    if_else(event.is_private, "P", "-"),
                ),
            ],
        );

        /* set color (encoded as BGR) for custom item drawing */
        let color = if rule.is_some() {
            Some(0xAAAAAA)
        } else if event.is_private {
            Some(0xCC0000)
        } else if event.is_injected {
            Some(0xCC00AA)
        } else {
            None
        };
        if let Some(color) = color {
            set_list_view_item_data(&self.list_view, self.list_view.len() - 1, color)
        };

        self.list_view.set_redraw(true);

        scroll_list_view_to_end(&self.list_view);
    }

    pub(crate) fn clear(&self) {
        self.list_view.clear()
    }

    fn handle_custom_draw(msg: u32, l_param: isize) -> Option<isize> {
        if msg != WM_NOTIFY {
            return None;
        }

        let hdr = unsafe { &*(l_param as *const NMHDR) };
        if hdr.code != NM_CUSTOMDRAW {
            return None;
        }

        let cd = unsafe { &mut *(l_param as *mut NMLVCUSTOMDRAW) };
        let stage = cd.nmcd.dwDrawStage;

        if stage == CDDS_PREPAINT {
            return Some(CDRF_NOTIFYITEMDRAW as isize);
        }

        if stage == CDDS_ITEMPREPAINT {
            let item_color = cd.nmcd.lItemlParam.0 as u32;
            if item_color != 0 {
                cd.clrText = COLORREF(item_color);
                return Some(CDRF_NEWFONT as isize);
            }
        }

        Some(CDRF_DODEFAULT as isize)
    }
}
