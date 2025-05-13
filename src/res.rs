extern crate native_windows_gui as nwg;
use nwg::EmbedResource;
use serde::Deserialize;
use std::fs;
use std::sync::LazyLock;
// pub(crate) const SOUND_GAME_LOCK_ON: &str = "./res/sound/game_lock_on.wav";
// pub(crate) const SOUND_GAME_LOCK_OFF: &str = "./res/sound/game_lock_off.wav";

#[derive(Deserialize)]
pub(crate) struct ResourceStrings {
    pub(crate) app_title: String,
    pub(crate) clear_log: String,
    pub(crate) profile: String,
    pub(crate) log: String,
    pub(crate) file: String,
    pub(crate) exit: String,
    pub(crate) tray_tip: String,
    pub(crate) open: String,
    pub(crate) enabled: String,
    pub(crate) logging_enabled: String,
    pub(crate) _logging_enabled_: String,
    pub(crate) _logging_disabled_: String,
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

pub(crate) struct Resources {
    pub(crate) embedded: EmbedResource,
}

impl Default for Resources {
    fn default() -> Self {
        Self {
            embedded: EmbedResource::load(None).unwrap(),
        }
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
