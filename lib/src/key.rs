use crate::error::KeyError;
use crate::key_code::ext_scan_code;
use crate::key_code::scan_code_name;
use crate::key_code::virtual_key_name;
use crate::key_error;
use log::error;
use std::fmt::{Debug, Display, Formatter};

macro_rules! define_keys {
    ($const_name:ident { $($variant:ident = ($index:expr, $name:literal, $vk:expr, $sc:expr, $sc_ext:expr)),* $(,)? }) => {
        #[repr(u8)]
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
        pub enum $const_name {
            $($variant = $index),*
        }

        impl $const_name {
            pub const fn vk(&self) -> u8 {
                match self {
                    $(Self::$variant => $vk),*
                }
            }

            pub const fn sc(&self) -> u8 {
                match self {
                    $(Self::$variant => $sc),*
                }
            }

            pub const fn sc_ext(&self) -> u16 {
                match self {
                    $(Self::$variant => ext_scan_code($sc, $sc_ext)),*
                }
            }

            pub const fn is_ext_sc(&self) -> bool {
                match self {
                    $(Self::$variant => $sc_ext),*
                }
            }

            pub const fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => $name),*
                }
            }

            pub const fn from_index(index: u8) -> Option<Self> {
                match index {
                    $($index => Some(Self::$variant)),*,
                    _ => None
                }
            }

            pub fn from_code(vk: u8, sc:u8, sc_ext:bool) -> Self {
                match (vk, sc, sc_ext) {
                    $(($vk, $sc, $sc_ext) => Self::$variant),*,
                    _ => {
                        error!("Unsupported key code: 0x{:02X} 0x{:02X} {}", vk, sc, sc_ext);
                        Self::Unassigned
                    }
                }
            }

            pub fn from_str(s: &str) -> Option<Self> {
                match s {
                    $($name => Some(Self::$variant)),*,
                    "" => Some(Self::Unassigned),
                    _ => None
                }
            }

        }
    };
}

impl Key {
    pub const fn sc_name(&self) -> &'static str {
        scan_code_name(self.sc(), self.is_ext_sc())
    }

    pub const fn vk_name(&self) -> &'static str {
        virtual_key_name(self.vk())
    }

    pub fn try_from_str(s: &str) -> Result<Self, KeyError> {
        Self::from_str(s).ok_or(key_error!("Unsupported key name: `{}`", s))
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

define_keys! {
    Key {
        Unassigned = (0, "UNASSIGNED", 0x00, 0x00, false),
        LeftButton = (1, "LEFT_BUTTON", 0x01, 0x00, false),
        RightButton = (2, "RIGHT_BUTTON", 0x02, 0x00, false),
        Break = (3, "BREAK", 0x03, 0x46, true),
        MiddleButton = (4, "MIDDLE_BUTTON", 0x04, 0x00, false),
        Xbutton1 = (5, "XBUTTON1", 0x05, 0x00, false),
        Xbutton2 = (6, "XBUTTON2", 0x06, 0x00, false),
        Backspace = (8, "BACKSPACE", 0x08, 0x0E, false),
        Tab = (9, "TAB", 0x09, 0x0F, false),
        NumClear = (12, "NUM_CLEAR", 0x0C, 0x4C, false),
        Enter = (13, "ENTER", 0x0D, 0x1C, false),
        Shift = (16, "SHIFT", 0x10, 0x2A, false),
        Ctrl = (17, "CTRL", 0x11, 0x1D, false),
        Menu = (18, "MENU", 0x12, 0x38, false),
        Pause = (19, "PAUSE", 0x13, 0x45, false),
        CapsLock = (20, "CAPS_LOCK", 0x14, 0x3A, false),
        Kana = (21, "KANA", 0x15, 0x00, false),
        ImeOn = (22, "IME_ON", 0x16, 0x00, false),
        Junja = (23, "JUNJA", 0x17, 0x00, false),
        Final = (24, "FINAL", 0x18, 0x00, false),
        Hanja = (25, "HANJA", 0x19, 0x00, false),
        ImeOff = (26, "IME_OFF", 0x1A, 0x00, false),
        Esc = (27, "ESC", 0x1B, 0x01, false),
        Convert = (28, "CONVERT", 0x1C, 0x00, false),
        NonConvert = (29, "NON_CONVERT", 0x1D, 0x00, false),
        Accept = (30, "ACCEPT", 0x1E, 0x00, false),
        ModeChange = (31, "MODE_CHANGE", 0x1F, 0x00, false),
        Space = (32, "SPACE", 0x20, 0x39, false),
        PageUp = (33, "PAGE_UP", 0x21, 0x49, true),
        PageDown = (34, "PAGE_DOWN", 0x22, 0x51, true),
        End = (35, "END", 0x23, 0x4F, true),
        Home = (36, "HOME", 0x24, 0x47, true),
        Left = (37, "LEFT", 0x25, 0x4B, true),
        Up = (38, "UP", 0x26, 0x48, true),
        Right = (39, "RIGHT", 0x27, 0x4D, true),
        Down = (40, "DOWN", 0x28, 0x50, true),
        Select = (41, "SELECT", 0x29, 0x00, false),
        Print = (42, "PRINT", 0x2A, 0x00, false),
        Execute = (43, "EXECUTE", 0x2B, 0x00, false),
        SysReq = (44, "SYS_REQ", 0x2C, 0x54, false),
        Insert = (45, "INSERT", 0x2D, 0x52, true),
        Delete = (46, "DELETE", 0x2E, 0x53, true),
        Help = (47, "HELP", 0x2F, 0x63, false),
        Digit0 = (48, "0", 0x30, 0x0B, false),
        Digit1 = (49, "1", 0x31, 0x02, false),
        Digit2 = (50, "2", 0x32, 0x03, false),
        Digit3 = (51, "3", 0x33, 0x04, false),
        Digit4 = (52, "4", 0x34, 0x05, false),
        Digit5 = (53, "5", 0x35, 0x06, false),
        Digit6 = (54, "6", 0x36, 0x07, false),
        Digit7 = (55, "7", 0x37, 0x08, false),
        Digit8 = (56, "8", 0x38, 0x09, false),
        Digit9 = (57, "9", 0x39, 0x0A, false),
        A = (65, "A", 0x41, 0x1E, false),
        B = (66, "B", 0x42, 0x30, false),
        C = (67, "C", 0x43, 0x2E, false),
        D = (68, "D", 0x44, 0x20, false),
        E = (69, "E", 0x45, 0x12, false),
        F = (70, "F", 0x46, 0x21, false),
        G = (71, "G", 0x47, 0x22, false),
        H = (72, "H", 0x48, 0x23, false),
        I = (73, "I", 0x49, 0x17, false),
        J = (74, "J", 0x4A, 0x24, false),
        K = (75, "K", 0x4B, 0x25, false),
        L = (76, "L", 0x4C, 0x26, false),
        M = (77, "M", 0x4D, 0x32, false),
        N = (78, "N", 0x4E, 0x31, false),
        O = (79, "O", 0x4F, 0x18, false),
        P = (80, "P", 0x50, 0x19, false),
        Q = (81, "Q", 0x51, 0x10, false),
        R = (82, "R", 0x52, 0x13, false),
        S = (83, "S", 0x53, 0x1F, false),
        T = (84, "T", 0x54, 0x14, false),
        U = (85, "U", 0x55, 0x16, false),
        V = (86, "V", 0x56, 0x2F, false),
        W = (87, "W", 0x57, 0x11, false),
        X = (88, "X", 0x58, 0x2D, false),
        Y = (89, "Y", 0x59, 0x15, false),
        Z = (90, "Z", 0x5A, 0x2C, false),
        LeftWin = (91, "LEFT_WIN", 0x5B, 0x5B, true),
        RightWin = (92, "RIGHT_WIN", 0x5C, 0x5C, true),
        Application = (93, "APPLICATION", 0x5D, 0x5D, true),
        Sleep = (95, "SLEEP", 0x5F, 0x5F, true),
        Num0 = (96, "NUM_0", 0x60, 0x52, false),
        Num1 = (97, "NUM_1", 0x61, 0x4F, false),
        Num2 = (98, "NUM_2", 0x62, 0x50, false),
        Num3 = (99, "NUM_3", 0x63, 0x51, false),
        Num4 = (100, "NUM_4", 0x64, 0x4B, false),
        Num5 = (101, "NUM_5", 0x65, 0x4C, false),
        Num6 = (102, "NUM_6", 0x66, 0x4D, false),
        Num7 = (103, "NUM_7", 0x67, 0x47, false),
        Num8 = (104, "NUM_8", 0x68, 0x48, false),
        Num9 = (105, "NUM_9", 0x69, 0x49, false),
        NumMul = (106, "NUM_MUL", 0x6A, 0x37, false),
        NumPlus = (107, "NUM_PLUS", 0x6B, 0x4E, false),
        Separator = (108, "SEPARATOR", 0x6C, 0x00, false),
        NumMinus = (109, "NUM_MINUS", 0x6D, 0x4A, false),
        NumDot = (110, "NUM_DOT", 0x6E, 0x53, false),
        NumDiv = (111, "NUM_DIV", 0x6F, 0x35, true),
        F1 = (112, "F1", 0x70, 0x3B, false),
        F2 = (113, "F2", 0x71, 0x3C, false),
        F3 = (114, "F3", 0x72, 0x3D, false),
        F4 = (115, "F4", 0x73, 0x3E, false),
        F5 = (116, "F5", 0x74, 0x3F, false),
        F6 = (117, "F6", 0x75, 0x40, false),
        F7 = (118, "F7", 0x76, 0x41, false),
        F8 = (119, "F8", 0x77, 0x42, false),
        F9 = (120, "F9", 0x78, 0x43, false),
        F10 = (121, "F10", 0x79, 0x44, false),
        F11 = (122, "F11", 0x7A, 0x57, false),
        F12 = (123, "F12", 0x7B, 0x58, false),
        F13 = (124, "F13", 0x7C, 0x64, false),
        F14 = (125, "F14", 0x7D, 0x65, false),
        F15 = (126, "F15", 0x7E, 0x66, false),
        F16 = (127, "F16", 0x7F, 0x67, false),
        F17 = (128, "F17", 0x80, 0x68, false),
        F18 = (129, "F18", 0x81, 0x69, false),
        F19 = (130, "F19", 0x82, 0x6A, false),
        F20 = (131, "F20", 0x83, 0x6B, false),
        F21 = (132, "F21", 0x84, 0x6C, false),
        F22 = (133, "F22", 0x85, 0x6D, false),
        F23 = (134, "F23", 0x86, 0x6E, false),
        F24 = (135, "F24", 0x87, 0x76, false),
        NumLock = (144, "NUM_LOCK", 0x90, 0x45, true),
        ScrollLock = (145, "SCROLL_LOCK", 0x91, 0x46, false),
        LeftShift = (160, "LEFT_SHIFT", 0xA0, 0x2A, false),
        RightShift = (161, "RIGHT_SHIFT", 0xA1, 0x36, true),
        LeftCtrl = (162, "LEFT_CTRL", 0xA2, 0x1D, false),
        RightCtrl = (163, "RIGHT_CTRL", 0xA3, 0x1D, true),
        LeftAlt = (164, "LEFT_ALT", 0xA4, 0x38, false),
        RightAlt = (165, "RIGHT_ALT", 0xA5, 0x38, true),
        BrowserBack = (166, "BROWSER_BACK", 0xA6, 0x6A, true),
        BrowserForward = (167, "BROWSER_FORWARD", 0xA7, 0x69, true),
        BrowserRefresh = (168, "BROWSER_REFRESH", 0xA8, 0x67, true),
        BrowserStop = (169, "BROWSER_STOP", 0xA9, 0x68, true),
        BrowserSearch = (170, "BROWSER_SEARCH", 0xAA, 0x00, true),
        BrowserFavorites = (171, "BROWSER_FAVORITES", 0xAB, 0x66, true),
        BrowserHome = (172, "BROWSER_HOME", 0xAC, 0x00, true),
        VolumeMute = (173, "VOLUME_MUTE", 0xAD, 0x00, true),
        VolumeDown = (174, "VOLUME_DOWN", 0xAE, 0x00, true),
        VolumeUp = (175, "VOLUME_UP", 0xAF, 0x00, true),
        MediaNextTrack = (176, "MEDIA_NEXT_TRACK", 0xB0, 0x00, true),
        MediaPrevTrack = (177, "MEDIA_PREV_TRACK", 0xB1, 0x00, true),
        MediaStop = (178, "MEDIA_STOP", 0xB2, 0x24, true),
        MediaPlayPause = (179, "MEDIA_PLAY_PAUSE", 0xB3, 0x00, true),
        LaunchMail = (180, "LAUNCH_MAIL", 0xB4, 0x00, true),
        LaunchMediaSelect = (181, "LAUNCH_MEDIA_SELECT", 0xB5, 0x6D, true),
        LaunchApp1 = (182, "LAUNCH_APP1", 0xB6, 0x00, true),
        LaunchApp2 = (183, "LAUNCH_APP2", 0xB7, 0x00, true),
        Semicolon = (186, "SEMICOLON", 0xBA, 0x27, false),
        Eq = (187, "EQ", 0xBB, 0x0D, false),
        Comma = (188, "COMMA", 0xBC, 0x33, false),
        Minus = (189, "MINUS", 0xBD, 0x0C, false),
        Dot = (190, "DOT", 0xBE, 0x34, false),
        Slash = (191, "SLASH", 0xBF, 0x35, false),
        Backtick = (192, "BACKTICK", 0xC0, 0x29, false),
        LeftBracket = (219, "LEFT_BRACKET", 0xDB, 0x1A, false),
        Backslash = (220, "BACKSLASH", 0xDC, 0x2B, false),
        RightBracket = (221, "RIGHT_BRACKET", 0xDD, 0x1B, false),
        Apostrophe = (222, "APOSTROPHE", 0xDE, 0x28, false),
        Oem8 = (223, "OEM_8", 0xDF, 0x00, false),
        Backslash2 = (226, "BACKSLASH_2", 0xE2, 0x56, false),
        ProcessKey = (229, "PROCESS_KEY", 0xE5, 0x00, false),
        WheelX = (241, "WHEEL_X", 0xF1, 0x00, true),
        WheelY = (243, "WHEEL_Y", 0xF3, 0x00, true),
        Attn = (246, "ATTN", 0xF6, 0x00, false),
        Crsel = (247, "CRSEL", 0xF7, 0x00, false),
        Exsel = (248, "EXSEL", 0xF8, 0x00, false),
        Ereof = (249, "EREOF", 0xF9, 0x5D, false),
        Play = (250, "PLAY", 0xFA, 0x00, false),
        Zoom = (251, "ZOOM", 0xFB, 0x62, false),
        Noname = (252, "NONAME", 0xFC, 0x00, false),
        Pa1 = (253, "PA1", 0xFD, 0x00, false),
        OemClear = (254, "OEM_CLEAR", 0xFE, 0x00, false),
        Brightness = (255, "BRIGHTNESS", 0xFF, 0x2B, true),

        _Esc_ = (193, "<ESC>", 0x00, 0x01, true),
        _Tab_ = (194, "<TAB>", 0x00, 0x0F, true),
        Brightness2 = (195, "BRIGHTNESS_2", 0x00, 0x2B, true),
        RightShift2 = (196, "RIGHT_SHIFT_2", 0x00, 0x36, true),
        Underscore = (197, "_", 0x00, 0x39, true),
        Plus = (198, "PLUS", 0x00, 0x4E, true),
        _00_ = (199, "<00>", 0x00, 0x54, true),
        NumEnter = (200, "NUM_ENTER", 0x0D, 0x1C, true),
        NumLock2 = (201, "NUM_LOCK_2", 0x13, 0x45, true),
        NumPageUp = (202, "NUM_PAGE_UP", 0x21, 0x49, false),
        NumPageDown = (203, "NUM_PAGE_DOWN", 0x22, 0x51, false),
        NumEnd = (204, "NUM_END", 0x23, 0x4F, false),
        NumHome = (205, "NUM_HOME", 0x24, 0x47, false),
        NumLeft = (206, "NUM_LEFT", 0x25, 0x4B, false),
        NumUp = (207, "NUM_UP", 0x26, 0x48, false),
        NumRight = (208, "NUM_RIGHT", 0x27, 0x4D, false),
        NumDown = (209, "NUM_DOWN", 0x28, 0x50, false),
        PrintScreen = (210, "PRINT_SCREEN", 0x2C, 0x37, true),
        NumInsert = (211, "NUM_INSERT", 0x2D, 0x52, false),
        NumDelete = (212, "NUM_DELETE", 0x2E, 0x53, false),
        FnBrowserSearch = (213, "FN_BROWSER_SEARCH", 0xAA, 0x65, true),
        FnBrowserHome = (214, "FN_BROWSER_HOME", 0xAC, 0x32, true),
        FnVolumeMute = (215, "FN_VOLUME_MUTE", 0xAD, 0x20, true),
        FnVolumeDown = (216, "FN_VOLUME_DOWN", 0xAE, 0x2E, true),
        FnVolumeUp = (217, "FN_VOLUME_UP", 0xAF, 0x30, true),
        FnMediaNextTrack = (218, "FN_MEDIA_NEXT_TRACK", 0xB0, 0x19, true),
        FnMediaPrevTrack = (224, "FN_MEDIA_PREV_TRACK", 0xB1, 0x10, true),
        FnMediaPlayPause = (225, "FN_MEDIA_PLAY_PAUSE", 0xB3, 0x22, true),
        FnLaunchMail = (227, "FN_LAUNCH_MAIL", 0xB4, 0x6C, true),
        FnLaunchApp1 = (228, "FN_LAUNCH_APP1", 0xB6, 0x6B, true),
        FnLaunchApp2 = (230, "FN_LAUNCH_APP2", 0xB7, 0x21, true),
    }
}

#[cfg(test)]
mod tests {
    use crate::key::Key;

    #[macro_export]
    macro_rules! key {
        ( $ text: literal) => {
            Key::from_str($text).unwrap()
        };
    }

    #[test]
    fn test_from_code() {
        assert_eq!(Key::from_code(0x41, 0x1E, false), Key::A);
    }

    #[test]
    fn test_from_index() {
        assert_eq!(Key::from_index(65), Some(Key::A));
    }

    #[test]
    fn test_index() {
        assert_eq!(Key::A as u8, 65);
    }

    #[test]
    fn test_from_str() {
        assert_eq!(Key::from_str("A"), Some(Key::A));
    }

    #[test]
    fn test_as_str() {
        assert_eq!(Key::A.as_str(), "A");
    }
}
