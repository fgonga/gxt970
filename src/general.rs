use crate::protocol::GENERAL_LEN;

pub const DPI_LEVELS: usize = 6;
const DPI_SLOTS: usize = 8;
const POLLING: usize = 2;
const XY_SPLIT: u8 = 0x80;
const LEVEL_STATE: usize = 3;
const DISABLED_MASK: usize = 4;
const DPI_CODES: usize = 5;
const DPI_COLORS: usize = 0x15;
const EFFECT: usize = 0x2D;

pub const DPI_VALUES: [u16; 59] = [
    200, 300, 400, 500, 600, 700, 800, 900, 1000, 1100, 1200, 1300, 1400, 1500, 1600, 1700, 1800, 1900, 2000, 2100, 2200,
    2300, 2400, 2500, 2600, 2700, 2800, 2900, 3000, 3100, 3200, 3300, 3400, 3500, 3600, 3700, 3800, 3900, 4000, 4100, 4200,
    4300, 4400, 4500, 4600, 4700, 4800, 4900, 5000, 5500, 6000, 6500, 7000, 7500, 8000, 8500, 9000, 9500, 10000,
];

pub const POLLING_RATES: [u16; 4] = [125, 250, 500, 1000];

pub type Rgb = [u8; 3];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Nibbles {
    None,
    High,
    Both,
}

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub at: usize,
    pub max: usize,
    pub count_at: Option<usize>,
}

#[derive(Clone, Copy, Debug)]
pub struct Effect {
    pub code: u8,
    pub name: &'static str,
    pub param: usize,
    pub nibbles: Nibbles,
    pub direction: Option<usize>,
    pub palette: Option<Palette>,
}

pub const EFFECTS: [Effect; 10] = [
    Effect { code: 1, name: "Onda arco-íris", param: 0x2E, nibbles: Nibbles::Both, direction: Some(0x2F), palette: None },
    Effect {
        code: 2,
        name: "Estático",
        param: 0x30,
        nibbles: Nibbles::High,
        direction: None,
        palette: Some(Palette { at: 0x31, max: 1, count_at: None }),
    },
    Effect {
        code: 3,
        name: "Respiração",
        param: 0x34,
        nibbles: Nibbles::Both,
        direction: None,
        palette: Some(Palette { at: 0x36, max: 7, count_at: Some(0x35) }),
    },
    Effect { code: 4, name: "Cauda", param: 0x4B, nibbles: Nibbles::Both, direction: None, palette: None },
    Effect { code: 5, name: "Néon", param: 0x4C, nibbles: Nibbles::Both, direction: None, palette: None },
    Effect {
        code: 6,
        name: "Arco-íris",
        param: 0x4D,
        nibbles: Nibbles::High,
        direction: None,
        palette: Some(Palette { at: 0x4E, max: 8, count_at: None }),
    },
    Effect {
        code: 7,
        name: "Cintilação",
        param: 0x6C,
        nibbles: Nibbles::Both,
        direction: None,
        palette: Some(Palette { at: 0x6D, max: 2, count_at: None }),
    },
    Effect { code: 8, name: "Aleatório", param: 0x73, nibbles: Nibbles::Both, direction: None, palette: None },
    Effect { code: 9, name: "Onda", param: 0x74, nibbles: Nibbles::Both, direction: None, palette: None },
    Effect { code: 0, name: "Desligado", param: 0, nibbles: Nibbles::None, direction: None, palette: None },
];

pub fn dpi_from_code(code: u8) -> Option<u16> {
    DPI_VALUES.get((code as usize).checked_sub(1)?).copied()
}

pub fn code_from_dpi(dpi: u16) -> Option<u8> {
    DPI_VALUES.iter().position(|&v| v == dpi).map(|i| i as u8 + 1)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct General(pub [u8; GENERAL_LEN]);

impl General {
    pub fn from_slice(bytes: &[u8]) -> Self {
        let mut out = [0u8; GENERAL_LEN];
        out.copy_from_slice(&bytes[..GENERAL_LEN]);
        Self(out)
    }

    pub fn polling_hz(&self) -> Option<u16> {
        POLLING_RATES.get(((self.0[POLLING] & 0x0F) as usize).checked_sub(1)?).copied()
    }

    pub fn set_polling_hz(&mut self, hz: u16) {
        if let Some(i) = POLLING_RATES.iter().position(|&r| r == hz) {
            self.0[POLLING] = self.0[POLLING] & 0xF0 | (i as u8 + 1);
        }
    }

    pub fn dpi_editable(&self) -> bool {
        self.0[POLLING] & XY_SPLIT == 0
    }

    pub fn dpi(&self, level: usize) -> Option<u16> {
        dpi_from_code(self.0[DPI_CODES + level])
    }

    pub fn set_dpi(&mut self, level: usize, dpi: u16) {
        if let Some(code) = code_from_dpi(dpi)
            && self.dpi_editable()
        {
            self.0[DPI_CODES + level] = code;
        }
    }

    pub fn level_enabled(&self, level: usize) -> bool {
        self.0[DISABLED_MASK] & (1 << level) == 0
    }

    pub fn enabled_count(&self) -> usize {
        (0..DPI_SLOTS).filter(|&l| self.level_enabled(l)).count()
    }

    pub fn current_level(&self) -> usize {
        let nth = (self.0[LEVEL_STATE] >> 4) as usize;
        (0..DPI_SLOTS).filter(|&l| self.level_enabled(l)).nth(nth.saturating_sub(1)).unwrap_or(0)
    }

    fn store_levels(&mut self, current: usize) {
        let enabled: Vec<usize> = (0..DPI_SLOTS).filter(|&l| self.level_enabled(l)).collect();
        let current = enabled.iter().position(|&l| l == current).unwrap_or(0) + 1;
        self.0[LEVEL_STATE] = (current as u8) << 4 | enabled.len() as u8 & 0x0F;
    }

    pub fn set_current_level(&mut self, level: usize) {
        if self.level_enabled(level) {
            self.store_levels(level);
        }
    }

    pub fn set_level_enabled(&mut self, level: usize, on: bool) {
        if !on && (self.enabled_count() <= 1 || !self.level_enabled(level)) {
            return;
        }
        let current = self.current_level();
        if on {
            self.0[DISABLED_MASK] &= !(1 << level);
        } else {
            self.0[DISABLED_MASK] |= 1 << level;
        }
        let current = if self.level_enabled(current) {
            current
        } else {
            (0..DPI_SLOTS).find(|&l| self.level_enabled(l)).unwrap_or(0)
        };
        self.store_levels(current);
    }

    pub fn dpi_color(&self, level: usize) -> Rgb {
        self.rgb(DPI_COLORS + level * 3)
    }

    pub fn set_dpi_color(&mut self, level: usize, color: Rgb) {
        self.set_rgb(DPI_COLORS + level * 3, color);
    }

    pub fn effect(&self) -> Option<&'static Effect> {
        EFFECTS.iter().find(|e| e.code == self.0[EFFECT])
    }

    pub fn effect_code(&self) -> u8 {
        self.0[EFFECT]
    }

    pub fn set_effect(&mut self, code: u8) {
        if EFFECTS.iter().any(|e| e.code == code) {
            self.0[EFFECT] = code;
        }
    }

    pub fn brightness(&self, e: &Effect) -> u8 {
        self.0[e.param] >> 4
    }

    pub fn set_brightness(&mut self, e: &Effect, value: u8) {
        if e.nibbles != Nibbles::None {
            self.0[e.param] = self.0[e.param] & 0x0F | (value & 0x0F) << 4;
        }
    }

    pub fn speed(&self, e: &Effect) -> u8 {
        self.0[e.param] & 0x0F
    }

    pub fn set_speed(&mut self, e: &Effect, value: u8) {
        if e.nibbles == Nibbles::Both {
            self.0[e.param] = self.0[e.param] & 0xF0 | value & 0x0F;
        }
    }

    pub fn reversed(&self, e: &Effect) -> Option<bool> {
        e.direction.map(|at| self.0[at] != 0)
    }

    pub fn set_reversed(&mut self, e: &Effect, on: bool) {
        if let Some(at) = e.direction {
            self.0[at] = on as u8;
        }
    }

    pub fn color_count(&self, p: &Palette) -> usize {
        p.count_at.map_or(p.max, |at| (self.0[at] as usize).clamp(1, p.max))
    }

    pub fn set_color_count(&mut self, p: &Palette, n: usize) {
        if let Some(at) = p.count_at {
            self.0[at] = n.clamp(1, p.max) as u8;
        }
    }

    pub fn color(&self, p: &Palette, i: usize) -> Rgb {
        self.rgb(p.at + i * 3)
    }

    pub fn set_color(&mut self, p: &Palette, i: usize, color: Rgb) {
        if i < p.max {
            self.set_rgb(p.at + i * 3, color);
        }
    }

    fn rgb(&self, at: usize) -> Rgb {
        [self.0[at], self.0[at + 1], self.0[at + 2]]
    }

    fn set_rgb(&mut self, at: usize, c: Rgb) {
        self.0[at..at + 3].copy_from_slice(&c);
    }

    pub fn diff(&self, other: &General) -> Vec<usize> {
        (0..GENERAL_LEN).filter(|&i| self.0[i] != other.0[i]).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::BLOCK_OFFSET;

    const BACKUP: &[u8] = include_bytes!("../fixtures/general-backup.bin");
    const REPORT: usize = 0x208;

    fn profile(n: usize) -> General {
        General::from_slice(&BACKUP[(n - 1) * REPORT + BLOCK_OFFSET..])
    }

    fn profile_1() -> General {
        profile(1)
    }

    #[test]
    fn decodes_profile_1() {
        let g = profile_1();
        assert_eq!(g.polling_hz(), Some(500));
        assert!(g.dpi_editable());
        let dpis: Vec<u16> = (0..DPI_LEVELS).filter_map(|l| g.dpi(l)).collect();
        assert_eq!(dpis, [800, 1600, 2400, 3200, 4000, 5000]);
        assert_eq!(g.enabled_count(), 6);
        assert_eq!(g.current_level(), 5);
        let colors: Vec<Rgb> = (0..DPI_LEVELS).map(|l| g.dpi_color(l)).collect();
        assert_eq!(colors, [[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 0], [0, 255, 255], [255, 0, 255]]);
        let e = g.effect().unwrap();
        assert_eq!(e.name, "Onda arco-íris");
        assert_eq!(g.speed(e), 1);
        let breathing = &EFFECTS[2];
        let p = breathing.palette.unwrap();
        assert_eq!(g.color_count(&p), 7);
        assert_eq!(g.color(&p, 0), [255, 0, 0]);
        assert_eq!(g.color(&p, 6), [255, 255, 255]);
        let stat = &EFFECTS[1];
        assert_eq!(g.brightness(stat), 4);
        assert_eq!(g.color(&stat.palette.unwrap(), 0), [255, 0, 0]);
    }

    #[test]
    fn edits_touch_only_their_bytes() {
        let original = profile_1();
        let mut g = original;
        g.set_dpi(2, 1200);
        assert_eq!(g.diff(&original), [DPI_CODES + 2]);
        let mut g = original;
        g.set_effect(2);
        g.set_brightness(&EFFECTS[1], 2);
        assert_eq!(g.diff(&original), [EFFECT, 0x30]);
        let mut g = original;
        g.set_level_enabled(5, false);
        assert_eq!(g.enabled_count(), 5);
        assert_eq!(g.current_level(), 0);
        assert_eq!(g.diff(&original), [LEVEL_STATE, DISABLED_MASK]);
        g.set_level_enabled(5, true);
        g.set_current_level(5);
        assert_eq!(g, original);
    }

    #[test]
    fn decodes_profile_2() {
        let g = profile(2);
        assert_eq!(g.polling_hz(), Some(250));
        assert_eq!(g.current_level(), 0);
        assert_eq!(g.dpi_color(0), [255, 0x46, 0]);
        assert_eq!(g.effect().unwrap().code, 1);
    }

    #[test]
    fn keeps_last_level() {
        let mut g = profile_1();
        for l in 0..DPI_LEVELS {
            g.set_level_enabled(l, false);
        }
        assert_eq!(g.enabled_count(), 1);
    }
}
