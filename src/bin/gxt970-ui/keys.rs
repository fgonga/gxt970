use egui::Key;
use gxt970::buttons::{MOD_ALT, MOD_CMD, MOD_CTRL, MOD_SHIFT};

pub fn hid_usage(key: Key) -> Option<u8> {
    let letters = [
        Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H, Key::I, Key::J, Key::K, Key::L, Key::M,
        Key::N, Key::O, Key::P, Key::Q, Key::R, Key::S, Key::T, Key::U, Key::V, Key::W, Key::X, Key::Y, Key::Z,
    ];
    if let Some(i) = letters.iter().position(|&k| k == key) {
        return Some(0x04 + i as u8);
    }
    let digits = [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6, Key::Num7, Key::Num8, Key::Num9, Key::Num0];
    if let Some(i) = digits.iter().position(|&k| k == key) {
        return Some(0x1E + i as u8);
    }
    let functions = [Key::F1, Key::F2, Key::F3, Key::F4, Key::F5, Key::F6, Key::F7, Key::F8, Key::F9, Key::F10, Key::F11, Key::F12];
    if let Some(i) = functions.iter().position(|&k| k == key) {
        return Some(0x3A + i as u8);
    }
    Some(match key {
        Key::Enter => 0x28,
        Key::Escape => 0x29,
        Key::Backspace => 0x2A,
        Key::Tab => 0x2B,
        Key::Space => 0x2C,
        Key::Minus => 0x2D,
        Key::Equals => 0x2E,
        Key::OpenBracket => 0x2F,
        Key::CloseBracket => 0x30,
        Key::Backslash => 0x31,
        Key::Semicolon => 0x33,
        Key::Quote => 0x34,
        Key::Backtick => 0x35,
        Key::Comma => 0x36,
        Key::Period => 0x37,
        Key::Slash => 0x38,
        Key::Insert => 0x49,
        Key::Home => 0x4A,
        Key::PageUp => 0x4B,
        Key::Delete => 0x4C,
        Key::End => 0x4D,
        Key::PageDown => 0x4E,
        Key::ArrowRight => 0x4F,
        Key::ArrowLeft => 0x50,
        Key::ArrowDown => 0x51,
        Key::ArrowUp => 0x52,
        _ => return None,
    })
}

pub fn modifier_bits(m: egui::Modifiers) -> u8 {
    let mut bits = 0;
    if m.ctrl {
        bits |= MOD_CTRL;
    }
    if m.shift {
        bits |= MOD_SHIFT;
    }
    if m.alt {
        bits |= MOD_ALT;
    }
    if m.mac_cmd {
        bits |= MOD_CMD;
    }
    bits
}
