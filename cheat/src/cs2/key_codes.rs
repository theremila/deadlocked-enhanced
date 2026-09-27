#![allow(unused)]
use egui::{Key, Modifiers, PointerButton};
use serde::{Deserialize, Serialize};
use strum::EnumIter;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, EnumIter, Serialize, Deserialize)]
pub enum KeyCode {
    None = 0,
    Num0 = 1,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Space = 66,
    Backspace,
    Tab,
    Escape = 71,
    Insert = 73,
    Delete,
    Home,
    End,
    LeftShift = 80,
    LeftAlt = 82,
    LeftControl = 84,
    MouseLeft = 317,
    MouseRight,
    MouseMiddle,
    Mouse4,
    Mouse5,
    MouseWheelUp,
    MouseWheelDown,
}

impl KeyCode {
    fn from_string(input: &str) -> Option<Self> {
        Some(match input {
            "Key0" => Self::Num0,
            "Key1" => Self::Num1,
            "Key2" => Self::Num2,
            "Key3" => Self::Num3,
            "Key4" => Self::Num4,
            "Key5" => Self::Num5,
            "Key6" => Self::Num6,
            "Key7" => Self::Num7,
            "Key8" => Self::Num8,
            "Key9" => Self::Num9,
            "KeyA" => Self::A,
            "KeyB" => Self::B,
            "KeyC" => Self::C,
            "KeyD" => Self::D,
            "KeyE" => Self::E,
            "KeyF" => Self::F,
            "KeyG" => Self::G,
            "KeyH" => Self::H,
            "KeyI" => Self::I,
            "KeyJ" => Self::J,
            "KeyK" => Self::K,
            "KeyL" => Self::L,
            "KeyM" => Self::M,
            "KeyN" => Self::N,
            "KeyO" => Self::O,
            "KeyP" => Self::P,
            "KeyQ" => Self::Q,
            "KeyR" => Self::R,
            "KeyS" => Self::S,
            "KeyT" => Self::T,
            "KeyU" => Self::U,
            "KeyV" => Self::V,
            "KeyW" => Self::W,
            "KeyX" => Self::X,
            "KeyY" => Self::Y,
            "KeyZ" => Self::Z,
            "Space" => Self::Space,
            "Backspace" => Self::Backspace,
            "Tab" => Self::Tab,
            "Escape" => Self::Escape,
            "Insert" => Self::Insert,
            "Delete" => Self::Delete,
            "Home" => Self::Home,
            "End" => Self::End,
            "MouseLeft" => KeyCode::MouseLeft,
            "MouseRight" => KeyCode::MouseRight,
            "MouseMiddle" => KeyCode::MouseMiddle,
            "Mouse4" => KeyCode::Mouse4,
            "Mouse5" => KeyCode::Mouse5,
            "MouseWheelUp" => KeyCode::MouseWheelUp,
            "MouseWheelDown" => KeyCode::MouseWheelDown,
            "None" => KeyCode::None,
            _ => return None,
        })
    }

    pub fn from_egui(key: Key) -> Option<Self> {
        Some(match key {
            Key::Num0 => Self::Num0,
            Key::Num1 => Self::Num1,
            Key::Num2 => Self::Num2,
            Key::Num3 => Self::Num3,
            Key::Num4 => Self::Num4,
            Key::Num5 => Self::Num5,
            Key::Num6 => Self::Num6,
            Key::Num7 => Self::Num7,
            Key::Num8 => Self::Num8,
            Key::Num9 => Self::Num9,
            Key::A => Self::A,
            Key::B => Self::B,
            Key::C => Self::C,
            Key::D => Self::D,
            Key::E => Self::E,
            Key::F => Self::F,
            Key::G => Self::G,
            Key::H => Self::H,
            Key::I => Self::I,
            Key::J => Self::J,
            Key::K => Self::K,
            Key::L => Self::L,
            Key::M => Self::M,
            Key::N => Self::N,
            Key::O => Self::O,
            Key::P => Self::P,
            Key::Q => Self::Q,
            Key::R => Self::R,
            Key::S => Self::S,
            Key::T => Self::T,
            Key::U => Self::U,
            Key::V => Self::V,
            Key::W => Self::W,
            Key::X => Self::X,
            Key::Y => Self::Y,
            Key::Z => Self::Z,
            Key::Space => Self::Space,
            Key::Backspace => Self::Backspace,
            Key::Tab => Self::Tab,
            Key::Escape => Self::Escape,
            Key::Insert => Self::Insert,
            Key::Delete => Self::Delete,
            Key::Home => Self::Home,
            Key::End => Self::End,
            _ => return None,
        })
    }

    pub fn from_egui_modifiers(modifiers: Modifiers) -> Option<Self> {
        Some(match modifiers {
            Modifiers::CTRL => Self::LeftControl,
            Modifiers::SHIFT => Self::LeftShift,
            Modifiers::ALT => Self::LeftAlt,
            _ => return None,
        })
    }

    pub fn from_egui_mouse(button: PointerButton) -> Self {
        match button {
            PointerButton::Primary => Self::MouseLeft,
            PointerButton::Secondary => Self::MouseRight,
            PointerButton::Middle => Self::MouseMiddle,
            PointerButton::Extra1 => Self::Mouse4,
            PointerButton::Extra2 => Self::Mouse5,
        }
    }

    pub fn usize(self) -> usize {
        self as usize
    }

    pub const fn to_evdev(self) -> Option<u16> {
        Some(match self {
            Self::A => 30,
            Self::B => 48,
            Self::C => 46,
            Self::D => 32,
            Self::E => 18,
            Self::F => 33,
            Self::G => 34,
            Self::H => 35,
            Self::I => 23,
            Self::J => 36,
            Self::K => 37,
            Self::L => 38,
            Self::M => 50,
            Self::N => 49,
            Self::O => 24,
            Self::P => 25,
            Self::Q => 16,
            Self::R => 19,
            Self::S => 31,
            Self::T => 20,
            Self::U => 22,
            Self::V => 47,
            Self::W => 17,
            Self::X => 45,
            Self::Y => 21,
            Self::Z => 44,
            Self::Num0 => 11,
            Self::Num1 => 2,
            Self::Num2 => 3,
            Self::Num3 => 4,
            Self::Num4 => 5,
            Self::Num5 => 6,
            Self::Num6 => 7,
            Self::Num7 => 8,
            Self::Num8 => 9,
            Self::Num9 => 10,
            Self::Space => 57,
            Self::Backspace => 14,
            Self::Tab => 15,
            Self::Escape => 1,
            Self::LeftShift => 42,
            Self::LeftControl => 29,
            Self::LeftAlt => 56,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MovementKeys {
    pub forward: u16,
    pub back: u16,
    pub left: u16,
    pub right: u16,
}

impl Default for MovementKeys {
    fn default() -> Self {
        Self {
            forward: 17, // KEY_W
            back: 31,    // KEY_S
            left: 30,    // KEY_A
            right: 32,   // KEY_D
        }
    }
}

impl MovementKeys {
    pub fn detect(input: &crate::cs2::input::Input) -> Self {
        if input.is_key_pressed(KeyCode::E) && !input.is_key_pressed(KeyCode::W) {
            return Self {
                forward: 18, // KEY_E
                back: 32,    // KEY_D
                left: 31,    // KEY_S
                right: 33,   // KEY_F
            };
        }
        if input.is_key_pressed(KeyCode::Z) && !input.is_key_pressed(KeyCode::W) {
            return Self {
                forward: 44, // KEY_Z
                back: 31,    // KEY_S
                left: 16,    // KEY_Q
                right: 32,   // KEY_D
            };
        }
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cs2::input::Input;

    #[test]
    fn evdev_scancode_mapping() {
        assert_eq!(KeyCode::W.to_evdev(), Some(17));
        assert_eq!(KeyCode::A.to_evdev(), Some(30));
        assert_eq!(KeyCode::S.to_evdev(), Some(31));
        assert_eq!(KeyCode::D.to_evdev(), Some(32));
        assert_eq!(KeyCode::Space.to_evdev(), Some(57));
        assert_eq!(KeyCode::None.to_evdev(), None);
    }

    #[test]
    fn movement_keys_detection() {
        let mut input = Input::new();
        assert_eq!(MovementKeys::detect(&input), MovementKeys::default());

        input.set_test_keys(&[KeyCode::E, KeyCode::D]);
        let detected = MovementKeys::detect(&input);
        assert_eq!(detected.forward, 18);
        assert_eq!(detected.back, 32);
        assert_eq!(detected.left, 31);
        assert_eq!(detected.right, 33);

        input.set_test_keys(&[KeyCode::W]);
        assert_eq!(MovementKeys::detect(&input), MovementKeys::default());
    }
}
