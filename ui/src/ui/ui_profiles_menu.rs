use crate::res::RESOURCES;
use crate::rs;
use keympostor::profile::Profiles;
use native_windows_gui::{ControlHandle, Event, Menu, MenuItem, NwgError, Window};

use crate::res::res_ids::IDS_PROFILE;
use crate::ui::App;

#[derive(Default)]
pub(crate) struct ProfilesMenu {
    menu: Menu,
    items: Vec<(MenuItem, String)>,
}

impl ProfilesMenu {
    pub(crate) fn build_ui(
        &mut self,
        parent: &Window,
        profiles: &Profiles,
    ) -> Result<(), NwgError> {
        Menu::builder()
            .parent(parent)
            .text(rs!(IDS_PROFILE))
            .build(&mut self.menu)?;

        self.build_items(profiles)?;

        Ok(())
    }

    fn build_items(&mut self, profiles: &Profiles) -> Result<(), NwgError> {
        self.items = vec![];

        for (name, profile) in &profiles.items {
            let mut item: MenuItem = MenuItem::default();
            MenuItem::builder()
                .parent(&self.menu)
                .text(&profile.title)
                .build(&mut item)?;

            self.items.push((item, name.clone()));
        }

        Ok(())
    }

    pub(crate) fn update_ui(&self, current_profile_name: &Option<String>) {
        for (item, item_profile_name) in &self.items {
            item.set_checked(match current_profile_name {
                Some(profile_name) => item_profile_name == profile_name,
                None => false,
            });
        }
    }

    pub(crate) fn handle_event(&self, app: &App, evt: Event, handle: ControlHandle) {
        match evt {
            Event::OnMenuItemSelected => {
                for (item, profile_name) in &self.items {
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
