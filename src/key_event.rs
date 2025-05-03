use crate::key_action::KeyAction;
use crate::key_code::{KeyCode, MAX_SC_CODE, MAX_VK_CODE};
use crate::key_hook::SELF_MARKER;
use log::warn;
use std::fmt::{Display, Formatter};
use windows::Win32::UI::WindowsAndMessaging::{KBDLLHOOKSTRUCT, LLKHF_INJECTED};

pub(crate) struct KeyboardEvent {
    kb: KBDLLHOOKSTRUCT,
    pub(crate) action: KeyAction,
    pub(crate) is_trigger: bool,
}

impl KeyboardEvent {
    pub(crate) fn from_kb(kb: KBDLLHOOKSTRUCT, kb_state: [u8; 256]) -> Self {
        Self {
            kb,
            action: KeyAction::from_kb(&kb, &kb_state),
            is_trigger: false,
        }
    }

    pub(crate) fn flags(&self) -> u32 {
        self.kb.flags.0
    }

    pub(crate) fn time(&self) -> u32 {
        self.kb.time
    }

    pub(crate) fn is_injected(&self) -> bool {
        self.kb.flags.contains(LLKHF_INJECTED)
    }

    pub(crate) fn is_private(&self) -> bool {
        self.is_injected() && (self.kb.dwExtraInfo as *const u8 == SELF_MARKER.as_ptr())
    }

    pub(crate) fn is_valid(&self) -> bool {
        if self.kb.scanCode > MAX_SC_CODE as u32 {
            warn!("Ignored invalid scancode: 0x{:04X}.", self.kb.scanCode);
            false
        } else if self.kb.vkCode > MAX_VK_CODE as u32 {
            warn!("Ignored invalid virtual key: 0x{:04X}.", self.kb.vkCode);
            false
        } else if self.kb.time == 0 {
            warn!("Ignored invalid time: {}.", self.kb.time);
            false
        } else {
            true
        }
    }
}

impl Display for KeyboardEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let scancode = self.action.key.scancode.unwrap();
        let virtual_key = self.action.key.virtual_key.unwrap();
        let modifiers = if let Some(m) = self.action.modifiers {
            &format!("{}", m)
        } else {
            "ANY"
        };
        write!(
            f,
            "T: {:>9} | {:18} | SC: {} | VK: {} | M: {} | F: {:08b} | {:8} | {:8}",
            self.time(),
            scancode.name(),
            scancode,
            virtual_key,
            modifiers,
            self.flags(),
            if self.is_injected() { "INJECTED" } else { "" },
            if self.is_private() { "PRIVATE" } else { "" }
        )
    }
}
