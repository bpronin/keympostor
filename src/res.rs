extern crate native_windows_gui as nwg;

use crate::ui_panic;
use nwg::EmbedResource;
use serde::Deserialize;
use std::cell::RefCell;
use std::fs;
use std::sync::LazyLock;
// pub(crate) const SOUND_GAME_LOCK_ON: &str = "./res/sound/game_lock_on.wav";
// pub(crate) const SOUND_GAME_LOCK_OFF: &str = "./res/sound/game_lock_off.wav";

#[derive(Deserialize)]
pub(crate) struct ResourceStrings {
    pub(crate) app_title: String,
    pub(crate) clear_log: String,
    pub(crate) profile: String,
    pub(crate) load_profile: String,
    pub(crate) log: String,
    pub(crate) file: String,
    pub(crate) exit: String,
    pub(crate) tray_tip: String,
    pub(crate) open: String,
    pub(crate) enabled: String,
    pub(crate) logging_enabled: String,
    pub(crate) _logging_enabled_: String,
    pub(crate) _logging_disabled_: String,
    pub(crate) load_profile_filter: String,
}

pub(crate) static RESOURCE_STRINGS: LazyLock<ResourceStrings> = LazyLock::new(|| {
    toml::from_str(
        &fs::read_to_string("./res/strings.toml").expect("Unable to read strings resources file"),
    )
    .expect("Unable to parse strings resources file")
});

#[macro_export]
macro_rules! rs {
    ($res_id:ident) => {
        &RESOURCE_STRINGS.$res_id
    };
}

thread_local! {
    static EMBED_RES: RefCell<EmbedResource> = RefCell::new(
        EmbedResource::load(None).expect("Unable to load embedded resources")
    )
}

pub(crate) static RES: Resources = Resources {};

pub(crate) struct Resources {}

impl Resources {
    pub(crate) fn get_icon(&self, icon_id: usize) -> nwg::Icon {
        let mut icon = nwg::Icon::default();

        EMBED_RES.with_borrow(|embed| {
            nwg::Icon::builder()
                .source_embed(Some(embed))
                .source_embed_id(icon_id)
                .strict(true)
                .size(Some((16, 16)))
                .build(&mut icon)
                .unwrap_or_else(|e| {
                    ui_panic!("{}", e);
                });
        });

        icon
    }
}

// #[cfg(test)]
// mod test {
//     use crate::res::SOUND_GAME_LOCK_OFF;
//     use crate::util::play_sound;
//
//     #[test]
//     fn test_play_sound() {
//         play_sound(SOUND_GAME_LOCK_OFF);
//     }
// }
