use crate::protocol::{ButtonMap, SLOT_COUNT};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseFn {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    DoubleClick,
    TripleClick,
    ScrollUp,
    ScrollDown,
    TiltLeft,
    TiltRight,
}

impl MouseFn {
    pub const ALL: [MouseFn; 11] = [
        MouseFn::Left,
        MouseFn::Right,
        MouseFn::Middle,
        MouseFn::Back,
        MouseFn::Forward,
        MouseFn::DoubleClick,
        MouseFn::TripleClick,
        MouseFn::ScrollUp,
        MouseFn::ScrollDown,
        MouseFn::TiltLeft,
        MouseFn::TiltRight,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MouseFn::Left => "Clique esquerdo",
            MouseFn::Right => "Clique direito",
            MouseFn::Middle => "Clique do meio",
            MouseFn::Back => "Recuar",
            MouseFn::Forward => "Avançar",
            MouseFn::DoubleClick => "Duplo clique",
            MouseFn::TripleClick => "Triplo clique",
            MouseFn::ScrollUp => "Roda para cima",
            MouseFn::ScrollDown => "Roda para baixo",
            MouseFn::TiltLeft => "Deslocar à esquerda",
            MouseFn::TiltRight => "Deslocar à direita",
        }
    }

    fn bytes(self) -> [u8; 4] {
        match self {
            MouseFn::Left => [0x11, 0x01, 0, 0],
            MouseFn::Right => [0x11, 0x02, 0, 0],
            MouseFn::Middle => [0x11, 0x04, 0, 0],
            MouseFn::Back => [0x11, 0x08, 0, 0],
            MouseFn::Forward => [0x11, 0x10, 0, 0],
            MouseFn::DoubleClick => [0x31, 0x01, 0x32, 0x02],
            MouseFn::TripleClick => [0x31, 0x01, 0x32, 0x03],
            MouseFn::ScrollUp => [0x12, 0x01, 0, 0],
            MouseFn::ScrollDown => [0x12, 0xFF, 0, 0],
            MouseFn::TiltLeft => [0x13, 0xFF, 0, 0],
            MouseFn::TiltRight => [0x13, 0x01, 0, 0],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DpiFn {
    Up,
    Down,
    Cycle,
}

impl DpiFn {
    pub const ALL: [DpiFn; 3] = [DpiFn::Up, DpiFn::Down, DpiFn::Cycle];

    pub fn label(self) -> &'static str {
        match self {
            DpiFn::Up => "DPI +",
            DpiFn::Down => "DPI −",
            DpiFn::Cycle => "Ciclo de DPI",
        }
    }

    fn bytes(self) -> [u8; 4] {
        match self {
            DpiFn::Up => [0x41, 0x01, 0, 0],
            DpiFn::Down => [0x41, 0x02, 0, 0],
            DpiFn::Cycle => [0x41, 0x00, 0, 0],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MediaFn {
    pub label: &'static str,
    pub mask: [u8; 3],
}

pub const MEDIA: [MediaFn; 18] = [
    MediaFn { label: "Reproduzir/Pausa", mask: [0x08, 0, 0] },
    MediaFn { label: "Parar", mask: [0x04, 0, 0] },
    MediaFn { label: "Faixa anterior", mask: [0x02, 0, 0] },
    MediaFn { label: "Faixa seguinte", mask: [0x01, 0, 0] },
    MediaFn { label: "Aumentar volume", mask: [0x40, 0, 0] },
    MediaFn { label: "Diminuir volume", mask: [0x80, 0, 0] },
    MediaFn { label: "Silenciar", mask: [0x10, 0, 0] },
    MediaFn { label: "Leitor multimédia", mask: [0, 0x01, 0] },
    MediaFn { label: "Correio electrónico", mask: [0, 0x10, 0] },
    MediaFn { label: "Calculadora", mask: [0, 0x20, 0] },
    MediaFn { label: "Explorador", mask: [0, 0x02, 0] },
    MediaFn { label: "Pesquisar", mask: [0, 0, 0x01] },
    MediaFn { label: "Página inicial", mask: [0, 0, 0x02] },
    MediaFn { label: "Navegador: recuar", mask: [0, 0, 0x04] },
    MediaFn { label: "Navegador: avançar", mask: [0, 0, 0x08] },
    MediaFn { label: "Navegador: parar", mask: [0, 0, 0x10] },
    MediaFn { label: "Navegador: actualizar", mask: [0, 0, 0x20] },
    MediaFn { label: "Favoritos", mask: [0, 0, 0x40] },
];

pub const MOD_CTRL: u8 = 0x01;
pub const MOD_SHIFT: u8 = 0x02;
pub const MOD_ALT: u8 = 0x04;
pub const MOD_CMD: u8 = 0x08;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Mouse(MouseFn),
    Dpi(DpiFn),
    Key { mods: u8, key: u8 },
    Media(MediaFn),
    Disabled,
    Empty,
    Other(u32),
}

impl Action {
    pub const LEFT_CLICK: u32 = 0x0111;
    pub const DISABLED: u32 = 0x0150;

    pub fn encode(self) -> u32 {
        let bytes = match self {
            Action::Mouse(f) => f.bytes(),
            Action::Dpi(f) => f.bytes(),
            Action::Key { mods, key } => [0x21, mods, key, 0],
            Action::Media(m) => [0x22, m.mask[0], m.mask[1], m.mask[2]],
            Action::Disabled => return Self::DISABLED,
            Action::Empty => return 0,
            Action::Other(v) => return v,
        };
        u32::from_le_bytes(bytes)
    }

    pub fn decode(value: u32) -> Action {
        if value == 0 {
            return Action::Empty;
        }
        if value == Self::DISABLED {
            return Action::Disabled;
        }
        if let Some(f) = MouseFn::ALL.into_iter().find(|f| u32::from_le_bytes(f.bytes()) == value) {
            return Action::Mouse(f);
        }
        if let Some(f) = DpiFn::ALL.into_iter().find(|f| u32::from_le_bytes(f.bytes()) == value) {
            return Action::Dpi(f);
        }
        let b = value.to_le_bytes();
        match b[0] {
            0x21 if b[3] == 0 && b[2] != 0 => Action::Key { mods: b[1], key: b[2] },
            0x22 => MEDIA
                .into_iter()
                .find(|m| m.mask == [b[1], b[2], b[3]])
                .map_or(Action::Other(value), Action::Media),
            _ => Action::Other(value),
        }
    }

    pub fn label(self) -> String {
        match self {
            Action::Mouse(f) => f.label().to_owned(),
            Action::Dpi(f) => f.label().to_owned(),
            Action::Key { mods, key } => key_combo_label(mods, key),
            Action::Media(m) => m.label.to_owned(),
            Action::Disabled => "Desactivado".to_owned(),
            Action::Empty => "Sem função".to_owned(),
            Action::Other(v) => match v.to_le_bytes()[0] {
                0x60 | 0x70 => format!("Macro ({v:08x})"),
                _ => format!("Outra função ({v:08x})"),
            },
        }
    }
}

pub fn key_combo_label(mods: u8, key: u8) -> String {
    let mut parts = Vec::new();
    for (bit, name) in [(MOD_CTRL, "Ctrl"), (MOD_ALT, "Alt"), (MOD_SHIFT, "Shift"), (MOD_CMD, "Cmd")] {
        if mods & bit != 0 {
            parts.push(name.to_owned());
        }
    }
    parts.push(key_name(key));
    parts.join(" + ")
}

pub fn key_name(usage: u8) -> String {
    match usage {
        0x04..=0x1D => ((b'A' + usage - 0x04) as char).to_string(),
        0x1E..=0x26 => ((b'1' + usage - 0x1E) as char).to_string(),
        0x27 => "0".into(),
        0x28 => "Enter".into(),
        0x29 => "Esc".into(),
        0x2A => "Retrocesso".into(),
        0x2B => "Tab".into(),
        0x2C => "Espaço".into(),
        0x2D => "-".into(),
        0x2E => "=".into(),
        0x2F => "[".into(),
        0x30 => "]".into(),
        0x31 => "\\".into(),
        0x33 => ";".into(),
        0x34 => "'".into(),
        0x35 => "`".into(),
        0x36 => ",".into(),
        0x37 => ".".into(),
        0x38 => "/".into(),
        0x3A..=0x45 => format!("F{}", usage - 0x39),
        0x49 => "Insert".into(),
        0x4A => "Home".into(),
        0x4B => "Page Up".into(),
        0x4C => "Delete".into(),
        0x4D => "End".into(),
        0x4E => "Page Down".into(),
        0x4F => "→".into(),
        0x50 => "←".into(),
        0x51 => "↓".into(),
        0x52 => "↑".into(),
        _ => format!("Tecla {usage:#04x}"),
    }
}

pub struct PhysicalButton {
    pub name: &'static str,
    pub badge: &'static str,
    pub slot: usize,
    pub pos: (f32, f32),
    pub factory: Action,
}

const fn key(usage: u8) -> Action {
    Action::Key { mods: 0, key: usage }
}

pub const BUTTONS: [PhysicalButton; 14] = [
    PhysicalButton { name: "Botão esquerdo", badge: "1", slot: 1, pos: (62.0, 103.0), factory: Action::Mouse(MouseFn::Left) },
    PhysicalButton { name: "Botão direito", badge: "2", slot: 2, pos: (167.0, 103.0), factory: Action::Mouse(MouseFn::Right) },
    PhysicalButton { name: "Roda", badge: "3", slot: 3, pos: (116.0, 125.0), factory: Action::Mouse(MouseFn::Middle) },
    PhysicalButton { name: "Botão superior 4", badge: "4", slot: 4, pos: (139.0, 172.0), factory: Action::Dpi(DpiFn::Up) },
    PhysicalButton { name: "Botão superior 5", badge: "5", slot: 5, pos: (139.0, 203.0), factory: Action::Dpi(DpiFn::Down) },
    PhysicalButton { name: "Lateral S1", badge: "S1", slot: 14, pos: (344.0, 173.0), factory: key(0x1E) },
    PhysicalButton { name: "Lateral S2", badge: "S2", slot: 7, pos: (351.0, 212.0), factory: key(0x1F) },
    PhysicalButton { name: "Lateral S3", badge: "S3", slot: 8, pos: (360.0, 246.0), factory: key(0x20) },
    PhysicalButton { name: "Lateral S4", badge: "S4", slot: 6, pos: (316.0, 185.0), factory: key(0x21) },
    PhysicalButton { name: "Lateral S5", badge: "S5", slot: 13, pos: (321.0, 224.0), factory: key(0x22) },
    PhysicalButton { name: "Lateral S6", badge: "S6", slot: 12, pos: (326.0, 262.0), factory: key(0x23) },
    PhysicalButton { name: "Lateral S7", badge: "S7", slot: 9, pos: (289.0, 197.0), factory: key(0x24) },
    PhysicalButton { name: "Lateral S8", badge: "S8", slot: 10, pos: (289.0, 239.0), factory: key(0x25) },
    PhysicalButton { name: "Lateral S9", badge: "S9", slot: 11, pos: (293.0, 278.0), factory: key(0x26) },
];

pub fn factory_map() -> ButtonMap {
    let mut map = [0u32; SLOT_COUNT];
    for b in &BUTTONS {
        map[b.slot - 1] = b.factory.encode();
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE_1: [u8; 56] = [
        0x11, 0x01, 0, 0, 0x11, 0x02, 0, 0, 0x11, 0x04, 0, 0, 0x41, 0x01, 0, 0, 0x41, 0x02, 0, 0, 0x21, 0, 0x21, 0, 0x21, 0,
        0x1f, 0, 0x21, 0, 0x20, 0, 0x21, 0, 0x24, 0, 0x21, 0, 0x25, 0, 0x21, 0, 0x26, 0, 0x21, 0, 0x23, 0, 0x21, 0, 0x22, 0,
        0x21, 0, 0x1e, 0,
    ];

    #[test]
    fn factory_map_matches_the_mouse() {
        let map = factory_map();
        for (i, chunk) in PROFILE_1.chunks(4).enumerate() {
            assert_eq!(map[i].to_le_bytes(), [chunk[0], chunk[1], chunk[2], chunk[3]], "slot {}", i + 1);
        }
        assert!(map[14..].iter().all(|&v| v == 0));
    }

    #[test]
    fn every_action_round_trips() {
        let mut actions: Vec<Action> = MouseFn::ALL.into_iter().map(Action::Mouse).collect();
        actions.extend(DpiFn::ALL.into_iter().map(Action::Dpi));
        actions.extend(MEDIA.into_iter().map(Action::Media));
        actions.extend([Action::Disabled, Action::Empty, Action::Key { mods: MOD_CTRL | MOD_SHIFT, key: 0x06 }]);
        for a in actions {
            assert_eq!(Action::decode(a.encode()), a);
        }
    }

    #[test]
    fn unknown_codes_are_kept() {
        assert_eq!(Action::decode(0x0101_c070), Action::Other(0x0101_c070));
        assert_eq!(Action::decode(0x0101_c070).encode(), 0x0101_c070);
    }
}
