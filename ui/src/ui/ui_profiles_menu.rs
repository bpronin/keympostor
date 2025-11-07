use crate::res::RESOURCES;
use crate::rs;
use keympostor::profile::{Profile};
use native_windows_gui::{ControlHandle, Event, Menu, MenuItem, NwgError, Window};
use std::fs;
use std::path::Path;

use crate::res::res_ids::{IDS_NO_PROFILE, IDS_PROFILE};
use crate::ui::App;

#[derive(Default)]
pub(crate) struct ProfilesMenu {
    menu: Menu,
    items: Vec<(MenuItem, Option<String>)>,
}

impl ProfilesMenu {
    pub(crate) fn build_ui(&mut self, parent: &Window) -> Result<(), NwgError> {
        Menu::builder()
            .parent(parent)
            .text(rs!(IDS_PROFILE))
            .build(&mut self.menu)?;

        self.build_items()?;

        Ok(())
    }

    fn build_items(&mut self) -> Result<(), NwgError> {
        self.items = vec![];

        let mut item: MenuItem = Default::default();
        MenuItem::builder()
            .parent(&self.menu)
            .text(rs!(IDS_NO_PROFILE))
            .build(&mut item)?;
        self.items.push((item, None));

        for (path, title) in list_profiles().unwrap() {
            let mut item: MenuItem = Default::default();
            MenuItem::builder()
                .parent(&self.menu)
                .text(&title)
                .build(&mut item)?;

            self.items.push((item, Some(path)));
        }

        Ok(())
    }

    pub(crate) fn update_ui(&self, current_profile: &Option<String>) {
        for (item, profile) in &self.items {
            item.set_checked(profile == current_profile);
        }
    }

    pub(crate) fn handle_event(&self, app: &App, evt: Event, handle: ControlHandle) {
        match evt {
            Event::OnMenuItemSelected => {
                for (item, path) in &self.items {
                    if item.handle == handle {
                        app.on_select_profile(path.clone());
                        break;
                    }
                }
            }
            _ => {}
        };
    }
}

fn list_profiles() -> anyhow::Result<Vec<(String, String)>> {
    let dir_entries = fs::read_dir(Path::new("profiles"))?;
    let mut result = vec![];
    for entry in dir_entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                let file_path = path.to_str().unwrap();
                let profile = Profile::load(file_path)?;
                result.push((file_path.to_string(), profile.title));
            }
        }
    }

    Ok(result)
}
