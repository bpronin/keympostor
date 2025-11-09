use crate::res::RESOURCES;
use crate::rs;
use keympostor::profile::Profiles;
use native_windows_gui::{ControlHandle, Event, Menu, MenuItem, NwgError, Window};

use crate::res::res_ids::{IDS_NO_PROFILE, IDS_PROFILE};
use crate::ui::App;

#[derive(Default)]
pub(crate) struct ProfilesMenu {
    menu: Menu,
    items: Vec<(MenuItem, Option<String>)>,
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

        let mut item: MenuItem = Default::default();
        MenuItem::builder()
            .parent(&self.menu)
            .text(rs!(IDS_NO_PROFILE))
            .build(&mut item)?;
        self.items.push((item, None));

        for (name, profile) in &profiles.items {
            let mut item: MenuItem = Default::default();
            MenuItem::builder()
                .parent(&self.menu)
                .text(&profile.title)
                .build(&mut item)?;

            self.items.push((item, Some(name.clone())));
        }

        Ok(())
    }

    pub(crate) fn update_ui(&self, current_profile: &Option<String>) {
        for (item, profile_name) in &self.items {
            item.set_checked(profile_name == current_profile);
        }
    }

    pub(crate) fn handle_event(&self, app: &App, evt: Event, handle: ControlHandle) {
        match evt {
            Event::OnMenuItemSelected => {
                for (item, profile_name) in &self.items {
                    if item.handle == handle {
                        app.on_select_profile(profile_name);
                        break;
                    }
                }
            }
            _ => {}
        };
    }
}
