use crate::layout::KeyTransformLayout;
use crate::rs;
use crate::ui::res::RESOURCES;
use crate::ui::res_ids::{IDS_ACTION, IDS_KEY};
use crate::ui::utils::{handle_list_view_custom_draw, set_list_view_item_data};
use keympostor::notify::KeyEventNotification;
use keympostor::rule::KeyTransformRule;
use native_windows_gui::{
    bind_raw_event_handler, ControlHandle, InsertListViewColumn, ListView, ListViewColumnFlags,
    ListViewExFlags, ListViewStyle, NwgError, Tab,
};
use std::cell::RefCell;
use windows::Win32::Foundation::COLORREF;

#[derive(Default)]
pub(crate) struct LayoutView {
    list_view: ListView,
    data: RefCell<Vec<KeyTransformRule>>,
}

impl LayoutView {
    pub(crate) fn view(&self) -> impl Into<ControlHandle> {
        &self.list_view
    }

    pub(crate) fn build(&mut self, parent: &Tab) -> Result<(), NwgError> {
        ListView::builder()
            .parent(parent)
            .list_style(ListViewStyle::Detailed)
            .ex_flags(ListViewExFlags::GRID | ListViewExFlags::FULL_ROW_SELECT)
            .build(&mut self.list_view)?;

        self.list_view.set_headers_enabled(true);

        self.list_view.insert_column(InsertListViewColumn {
            index: Some(0),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(300),
            text: Some(rs!(IDS_KEY).into()),
        });

        self.list_view.insert_column(InsertListViewColumn {
            index: Some(1),
            fmt: Some(ListViewColumnFlags::LEFT),
            width: Some(300),
            text: Some(rs!(IDS_ACTION).into()),
        });

        bind_raw_event_handler(
            &parent.handle,
            0x10002,
            move |_hwnd, msg, _w_param, l_param| {
                handle_list_view_custom_draw(msg, l_param, |cd| {
                    let item_color = cd.nmcd.lItemlParam.0 as u32;
                    if item_color != 0 {
                        cd.clrText = COLORREF(item_color);
                        return true;
                    }
                    return false;
                })
            },
        )?;

        Ok(())
    }

    pub(crate) fn update_ui(&self, layout: Option<&KeyTransformLayout>) {
        self.list_view.clear();

        let mut data = self.data.borrow_mut();
        data.clear();
        if let Some(layout) = layout {
            if let Some(rules) = &layout.rules {
                for (i, rule) in rules.iter().enumerate() {
                    data.push(rule.clone());
                    self.list_view.insert_items_row(
                        Some(i as i32),
                        &[rule.trigger.to_string(), rule.actions.to_string()],
                    );
                }
            }
        }
    }

    pub(crate) fn on_key_event(&self, notification: &KeyEventNotification) {
        if let Some(rule) = &notification.rule {
            self.list_view.set_redraw(false);

            for (i, item_rule) in self.data.borrow().iter().enumerate() {
                /* set color (encoded as BGR) for custom item drawing */
                let color = if rule == item_rule {
                    0x0000CC
                } else {
                    0x000000
                };
                set_list_view_item_data(&self.list_view, i, color)
            }

            self.list_view.set_redraw(true);
        }
    }
}
