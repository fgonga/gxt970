use craft_ui::theme::WidgetPalette;
use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, Visuals};
use std::sync::Arc;

pub struct Tokens;

impl Tokens {
    pub const TITLEBAR: Color32 = Color32::from_rgb(0x1B, 0x1B, 0x1E);
    pub const CHROME: Color32 = Color32::from_rgb(0x26, 0x26, 0x2A);
    pub const PASTEBOARD: Color32 = Color32::from_rgb(0x19, 0x19, 0x1C);
    pub const CARD: Color32 = Color32::from_rgb(0x2E, 0x2E, 0x33);
    pub const BORDER: Color32 = Color32::from_rgb(0x3C, 0x3C, 0x43);
    pub const DIVIDER: Color32 = Color32::from_rgb(0x33, 0x33, 0x39);
    pub const TEXT: Color32 = Color32::from_rgb(0xEC, 0xEC, 0xEF);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(0xAE, 0xAE, 0xB6);
    pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x97, 0x97, 0x9E);
    pub const HOVER: Color32 = Color32::from_rgb(0x34, 0x34, 0x3A);
    pub const PRESSED: Color32 = Color32::from_rgb(0x3E, 0x3E, 0x45);
    pub const SELECTED: Color32 = Color32::from_rgb(0x23, 0x3A, 0x63);
    pub const ACCENT: Color32 = Color32::from_rgb(0x4B, 0x8B, 0xF5);
    pub const ACCENT_TEXT: Color32 = Color32::from_rgb(0x8C, 0xB6, 0xFA);
    pub const ACCENT_SOFT: Color32 = Color32::from_rgb(0x24, 0x36, 0x57);
    pub const FIELD: Color32 = Color32::from_rgb(0x1E, 0x1E, 0x22);
    pub const BRAND: Color32 = Color32::from_rgb(0xE5, 0x2B, 0x2B);
    pub const OK: Color32 = Color32::from_rgb(0x3C, 0xC8, 0x7A);
    pub const RADIUS: u8 = 6;
}

pub fn install(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    for (name, bytes) in [
        ("Inter", &include_bytes!("../../../assets/fonts/Inter-Regular.ttf")[..]),
        ("Inter-Medium", &include_bytes!("../../../assets/fonts/Inter-Medium.ttf")[..]),
        ("Inter-SemiBold", &include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf")[..]),
    ] {
        fonts.font_data.insert(name.to_owned(), Arc::new(FontData::from_static(bytes)));
    }
    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "Inter".to_owned());
    let fallback = fonts.families[&FontFamily::Proportional].clone();
    for (family, primary) in [("medium", "Inter-Medium"), ("semibold", "Inter-SemiBold")] {
        let mut stack = vec![primary.to_owned()];
        stack.extend(fallback.iter().cloned());
        fonts.families.insert(FontFamily::Name(family.into()), stack);
    }
    ctx.set_fonts(fonts);

    ctx.set_theme(egui::Theme::Dark);
    let mut v = Visuals::dark();
    v.panel_fill = Tokens::CHROME;
    v.window_fill = Tokens::CARD;
    v.window_stroke = Stroke::new(1.0, Tokens::BORDER);
    v.extreme_bg_color = Tokens::FIELD;
    v.faint_bg_color = Tokens::HOVER;
    v.selection.bg_fill = Tokens::ACCENT_SOFT;
    v.selection.stroke = Stroke::new(1.0, Tokens::ACCENT);
    v.hyperlink_color = Tokens::ACCENT_TEXT;
    v.override_text_color = Some(Tokens::TEXT);
    v.window_corner_radius = CornerRadius::same(10);
    v.menu_corner_radius = CornerRadius::same(8);
    v.popup_shadow = egui::Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha(90) };
    WidgetPalette {
        noninteractive: Tokens::CHROME,
        inactive: Tokens::FIELD,
        hovered: Tokens::HOVER,
        active: Tokens::PRESSED,
        open: Tokens::HOVER,
    }
    .apply_to(&mut v.widgets, CornerRadius::same(Tokens::RADIUS), Stroke::new(1.0, Tokens::TEXT));
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Tokens::DIVIDER);
    v.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    v.widgets.inactive.bg_stroke = Stroke::NONE;
    v.widgets.hovered.bg_stroke = Stroke::NONE;
    v.text_options.subpixel_binning = false;
    ctx.set_visuals(v);
    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = egui::vec2(8.0, 6.0);
        s.spacing.button_padding = egui::vec2(10.0, 5.0);
        s.spacing.menu_margin = egui::Margin::same(6);
        s.spacing.scroll.bar_width = 8.0;
        s.spacing.scroll.floating = true;
        s.text_styles.insert(egui::TextStyle::Body, regular(13.0));
        s.text_styles.insert(egui::TextStyle::Button, regular(13.0));
        s.text_styles.insert(egui::TextStyle::Small, regular(11.0));
        s.text_styles.insert(egui::TextStyle::Heading, semibold(17.0));
        s.interaction.tooltip_delay = 0.35;
    });
}

pub fn regular(size: f32) -> FontId {
    FontId::proportional(size)
}

pub fn medium(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("medium".into()))
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name("semibold".into()))
}
