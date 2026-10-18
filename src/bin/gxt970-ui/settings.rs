use crate::app::{App, Section};
use crate::theme::{self, Tokens};
use egui::{Align, Align2, Color32, CornerRadius, Frame, Layout, Margin, Pos2, Rect, RichText, Stroke, Ui, vec2};
use gxt970::general::{DPI_LEVELS, DPI_VALUES, EFFECTS, Effect, Nibbles, POLLING_RATES, Rgb};
use std::ops::Range;

const LEVEL_MAX: u8 = 4;

fn range(section: Section) -> Range<usize> {
    match section {
        Section::Performance => 2..3,
        Section::Dpi => 3..0x2D,
        Section::Lighting => 0x2D..gxt970::protocol::GENERAL_LEN,
        Section::Buttons => 0..0,
    }
}

pub fn section_dirty(app: &App, section: Section) -> bool {
    app.settings_device.is_some_and(|d| d.0[range(section)] != app.settings.0[range(section)])
}

fn heading(ui: &mut Ui, title: &str, hint: &str) {
    ui.label(RichText::new(title).font(theme::semibold(16.0)));
    ui.label(RichText::new(hint).color(Tokens::TEXT_FAINT).size(12.0));
    ui.add_space(10.0);
}

fn card<R>(ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(Tokens::CARD)
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

fn option(ui: &mut Ui, label: &str, on: bool, width: f32) -> bool {
    let b = egui::Button::new(label)
        .selected(on)
        .fill(if on { Tokens::SELECTED } else { Tokens::FIELD })
        .stroke(Stroke::new(1.0, if on { Tokens::ACCENT } else { Tokens::DIVIDER }))
        .min_size(vec2(width, 28.0));
    ui.add(b).clicked()
}

fn note(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).color(Tokens::TEXT_FAINT).size(11.5));
}

fn swatch(ui: &mut Ui, color: &mut Rgb) -> bool {
    egui::color_picker::color_edit_button_srgb(ui, color).changed()
}

pub fn dpi(app: &mut App, ui: &mut Ui) {
    heading(ui, "Níveis de DPI", "O botão de DPI do rato percorre os níveis activos.");
    let mut g = app.settings;
    if !g.dpi_editable() {
        note(ui, "Este perfil usa DPI separado para X e Y, que esta versão ainda não edita.");
        return;
    }
    let current = g.current_level();
    card(ui, |ui| {
        egui::Grid::new("dpi_levels").num_columns(4).spacing(vec2(8.0, 10.0)).show(ui, |ui| {
            ui.label(RichText::new("Nível").size(11.5).color(Tokens::TEXT_FAINT));
            ui.label(RichText::new("Cor").size(11.5).color(Tokens::TEXT_FAINT));
            ui.label(RichText::new("DPI").size(11.5).color(Tokens::TEXT_FAINT));
            ui.label(RichText::new("Em uso").size(11.5).color(Tokens::TEXT_FAINT));
            ui.end_row();
            for level in 0..DPI_LEVELS {
                let mut enabled = g.level_enabled(level);
                if ui.checkbox(&mut enabled, format!("{}", level + 1)).changed() {
                    g.set_level_enabled(level, enabled);
                }
                ui.add_enabled_ui(enabled, |ui| {
                    let mut color = g.dpi_color(level);
                    if swatch(ui, &mut color) {
                        g.set_dpi_color(level, color);
                    }
                });
                ui.add_enabled_ui(enabled, |ui| {
                    let value = g.dpi(level).unwrap_or(DPI_VALUES[0]);
                    egui::ComboBox::from_id_salt(("dpi", level))
                        .width(150.0)
                        .height(260.0)
                        .selected_text(RichText::new(format!("{value} DPI")).font(theme::medium(12.5)))
                        .show_ui(ui, |ui| {
                            for dpi in DPI_VALUES {
                                if ui.selectable_label(dpi == value, format!("{dpi} DPI")).clicked() {
                                    g.set_dpi(level, dpi);
                                }
                            }
                        });
                });
                ui.add_enabled_ui(enabled, |ui| {
                    if ui.radio(current == level, "").clicked() {
                        g.set_current_level(level);
                    }
                });
                ui.end_row();
            }
        });
    });
    app.settings = g;
    ui.add_space(8.0);
    note(ui, "Tem de ficar pelo menos um nível activo. A cor de cada nível é a que o rato mostra ao mudar de DPI.");
}

pub fn performance(app: &mut App, ui: &mut Ui) {
    heading(ui, "Taxa de actualização", "Quantas vezes por segundo o rato envia a posição.");
    let g = &mut app.settings;
    let current = g.polling_hz();
    card(ui, |ui| {
        let w = ((ui.available_width() - 6.0) / 2.0).floor();
        egui::Grid::new("polling").num_columns(2).spacing(vec2(6.0, 6.0)).show(ui, |ui| {
            for (i, hz) in POLLING_RATES.into_iter().enumerate() {
                if option(ui, &format!("{hz} Hz"), current == Some(hz), w) {
                    g.set_polling_hz(hz);
                }
                if i % 2 == 1 {
                    ui.end_row();
                }
            }
        });
    });
    ui.add_space(8.0);
    note(ui, "1000 Hz dá a resposta mais rápida e gasta um pouco mais de CPU.");
}

fn level_slider(ui: &mut Ui, label: &str, value: u8) -> Option<u8> {
    let mut v = value.min(LEVEL_MAX);
    let mut changed = None;
    ui.horizontal(|ui| {
        ui.add_sized(vec2(68.0, 20.0), egui::Label::new(RichText::new(label).color(Tokens::TEXT_MUTED)));
        ui.spacing_mut().slider_width = ui.available_width() - 48.0;
        if ui.add(egui::Slider::new(&mut v, 0..=LEVEL_MAX)).changed() {
            changed = Some(v);
        }
    });
    changed
}

fn effect_params(g: &mut gxt970::general::General, e: &'static Effect, ui: &mut Ui) {
    if e.nibbles != Nibbles::None
        && let Some(v) = level_slider(ui, "Brilho", g.brightness(e))
    {
        g.set_brightness(e, v);
    }
    if e.nibbles == Nibbles::Both
        && let Some(v) = level_slider(ui, "Velocidade", g.speed(e))
    {
        g.set_speed(e, v);
    }
    if let Some(mut reversed) = g.reversed(e)
        && ui.checkbox(&mut reversed, "Sentido inverso").changed()
    {
        g.set_reversed(e, reversed);
    }
    if let Some(p) = e.palette {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("Cores").color(Tokens::TEXT_MUTED));
            if p.count_at.is_some() {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let mut n = g.color_count(&p);
                    if ui.add(egui::DragValue::new(&mut n).range(1..=p.max).suffix(" cores")).changed() {
                        g.set_color_count(&p, n);
                    }
                });
            }
        });
        ui.horizontal_wrapped(|ui| {
            for i in 0..g.color_count(&p) {
                let mut c = g.color(&p, i);
                if swatch(ui, &mut c) {
                    g.set_color(&p, i, c);
                }
            }
        });
    }
}

pub fn lighting(app: &mut App, ui: &mut Ui) {
    heading(ui, "Iluminação", "Efeito do LED deste perfil.");
    let g = &mut app.settings;
    let code = g.effect_code();
    card(ui, |ui| {
        let w = ((ui.available_width() - 6.0) / 2.0).floor();
        egui::Grid::new("effects").num_columns(2).spacing(vec2(6.0, 6.0)).show(ui, |ui| {
            for (i, e) in EFFECTS.iter().enumerate() {
                if option(ui, e.name, e.code == code, w) {
                    g.set_effect(e.code);
                }
                if i % 2 == 1 {
                    ui.end_row();
                }
            }
        });
    });
    ui.add_space(10.0);
    match g.effect() {
        Some(e) if e.code != 0 => {
            card(ui, |ui| {
                ui.label(RichText::new(e.name).font(theme::semibold(14.0)));
                ui.add_space(6.0);
                effect_params(g, e, ui);
            });
            ui.add_space(8.0);
            note(ui, "A escala de brilho e velocidade (0 a 4) ainda não foi confirmada no rato.");
        }
        Some(_) => note(ui, "O LED fica apagado neste perfil."),
        None => note(ui, &format!("Efeito desconhecido (código {code}); fica como está até escolheres outro.")),
    }
}

pub fn glow_tint(app: &App) -> Option<Rgb> {
    let g = &app.settings;
    app.settings_device?;
    match app.section {
        Section::Dpi => Some(g.dpi_color(g.current_level())),
        Section::Lighting => {
            let e = g.effect()?;
            e.palette.map(|p| g.color(&p, 0))
        }
        _ => None,
    }
}

pub fn stage_overlay(app: &App, ui: &mut Ui, rect: Rect) {
    let g = &app.settings;
    let painter = ui.painter();
    let caption = match app.section {
        Section::Dpi => {
            let levels: Vec<usize> = (0..DPI_LEVELS).filter(|&l| g.level_enabled(l)).collect();
            let y = rect.bottom() - 58.0;
            let gap = 12.0;
            let w = 64.0;
            let total = levels.len() as f32 * w + (levels.len().saturating_sub(1)) as f32 * gap;
            let mut x = rect.center().x - total / 2.0;
            for &l in &levels {
                let r = Rect::from_min_size(Pos2::new(x, y - 16.0), vec2(w, 30.0));
                let [cr, cg, cb] = g.dpi_color(l);
                let on = l == g.current_level();
                painter.rect(
                    r,
                    CornerRadius::same(6),
                    if on { Tokens::CARD } else { Tokens::FIELD },
                    Stroke::new(if on { 1.5 } else { 1.0 }, if on { Color32::from_rgb(cr, cg, cb) } else { Tokens::DIVIDER }),
                    egui::StrokeKind::Inside,
                );
                painter.circle_filled(Pos2::new(r.left() + 10.0, r.center().y), 3.5, Color32::from_rgb(cr, cg, cb));
                painter.text(
                    Pos2::new(r.center().x + 5.0, r.center().y),
                    Align2::CENTER_CENTER,
                    g.dpi(l).map_or("?".into(), |d| d.to_string()),
                    theme::medium(12.0),
                    if on { Tokens::TEXT } else { Tokens::TEXT_MUTED },
                );
                x += w + gap;
            }
            format!("DPI em uso: {}", g.dpi(g.current_level()).map_or("?".into(), |d| d.to_string()))
        }
        Section::Lighting => format!("Efeito: {}", g.effect().map_or("desconhecido", |e| e.name)),
        Section::Performance => format!("Taxa: {}", g.polling_hz().map_or("?".into(), |h| format!("{h} Hz"))),
        Section::Buttons => String::new(),
    };
    painter.text(Pos2::new(rect.center().x, rect.bottom() - 24.0), Align2::CENTER_CENTER, caption, theme::medium(14.0), Tokens::TEXT);
}
