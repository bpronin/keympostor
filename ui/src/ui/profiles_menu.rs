use std::cell::RefCell;
use crate::res::RESOURCES;
use crate::rs;
use keympostor::profile::Profiles;
use native_windows_gui as nwg;

use crate::res::res_ids::IDS_PROFILE;
use crate::ui::App;

#[derive(Default)]
pub(crate) struct ProfilesMenu {
    menu: nwg::Menu,
    items: RefCell<Vec<(nwg::MenuItem, String)>>,
}

impl ProfilesMenu {

    pub(crate) fn build_ui(
        &mut self,
        parent: &nwg::Window,
    ) -> Result<(), nwg::NwgError> {
        nwg::Menu::builder()
            .parent(parent)
            .text(rs!(IDS_PROFILE))
            .build(&mut self.menu)?;

        Ok(())
    }

    pub(crate) fn build_items(&self, profiles: &Profiles) -> Result<(), nwg::NwgError> {
        let mut items = vec![];

        for profile in &profiles.0 {
            let mut item: nwg::MenuItem = nwg::MenuItem::default();
            nwg::MenuItem::builder()
                .parent(&self.menu)
                .text(&profile.title)
                .build(&mut item)?;

            items.push((item, profile.name.clone()));
        }

        self.items.replace(items);

        Ok(())
    }

    pub(crate) fn update_ui(&self, current_profile_name: &Option<String>) {
        for (item, item_profile_name) in self.items.borrow().iter() {
            item.set_checked(match current_profile_name {
                Some(profile_name) => item_profile_name == profile_name,
                None => false,
            });
        }
    }

    pub(crate) fn handle_event(&self, app: &App, evt: nwg::Event, handle: nwg::ControlHandle) {
        match evt {
            nwg::Event::OnMenuItemSelected => {
                for (item, profile_name) in self.items.borrow().iter() {
                    if item.handle == handle {
                        app.on_select_profile(&Some(profile_name.to_string()));
                        break;
                    }
                }
            }
            _ => {}
        };
    }
}
