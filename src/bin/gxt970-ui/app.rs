use crate::keys;
use crate::theme::{self, Tokens};
use crate::worker::{Reply, Request, Snapshot, Worker};
use craft_ui::tabs::Tab;
use egui::{
    Align, Align2, Color32, CornerRadius, Frame, Id, Layout, Margin, Pos2, Rect, RichText, Sense, Stroke,
    TextureHandle, Ui, Vec2, vec2,
};
use gxt970::buttons::{Action, BUTTONS, DpiFn, MEDIA, MOD_ALT, MOD_CMD, MOD_CTRL, MOD_SHIFT, MouseFn, factory_map};
use gxt970::general::General;
use gxt970::protocol::{ButtonMap, GENERAL_LEN};

const MOUSE_PNG: &[u8] = include_bytes!("../../../assets/mouse.png");
const MOUSE_SIZE: Vec2 = vec2(418.0, 440.0);
const BADGE_RADIUS: f32 = 13.0;
const PROFILES: [u8; 3] = [1, 2, 3];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Category {
    Mouse,
    Dpi,
    Keyboard,
    Media,
    Other,
}

impl Category {
    const ALL: [Category; 5] = [Category::Mouse, Category::Dpi, Category::Keyboard, Category::Media, Category::Other];

    fn label(self) -> &'static str {
        match self {
            Category::Mouse => "Rato",
            Category::Dpi => "DPI",
            Category::Keyboard => "Teclado",
            Category::Media => "Multimédia",
            Category::Other => "Outras",
        }
    }

    fn of(action: Action) -> Category {
        match action {
            Action::Mouse(_) => Category::Mouse,
            Action::Dpi(_) => Category::Dpi,
            Action::Key { .. } => Category::Keyboard,
            Action::Media(_) => Category::Media,
            _ => Category::Other,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Section {
    Buttons,
    Dpi,
    Lighting,
    Performance,
}

impl Section {
    const ALL: [Section; 4] = [Section::Buttons, Section::Dpi, Section::Lighting, Section::Performance];

    fn label(self) -> &'static str {
        match self {
            Section::Buttons => "Botões",
            Section::Dpi => "DPI",
            Section::Lighting => "Iluminação",
            Section::Performance => "Desempenho",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tone {
    Info,
    Ok,
    Error,
}

pub struct App {
    worker: Option<Worker>,
    texture: Option<TextureHandle>,
    themed: bool,
    show_all_rings: bool,
    connected: bool,
    profiles: Vec<u8>,
    profile: u8,
    active: Option<u8>,
    pub(crate) section: Section,
    device: Option<ButtonMap>,
    draft: ButtonMap,
    pub(crate) settings_device: Option<General>,
    pub(crate) settings: General,
    selected: usize,
    category: Category,
    capturing: bool,
    just_captured: bool,
    applying: bool,
    busy: bool,
    status: (Tone, String),
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install(&cc.egui_ctx);
        let worker = Worker::spawn(cc.egui_ctx.clone());
        worker.send(Request::Connect);
        Self { themed: true, ..Self::with_worker(Some(worker)) }
    }

    fn with_worker(worker: Option<Worker>) -> Self {
        Self {
            worker,
            texture: None,
            themed: false,
            show_all_rings: false,
            connected: false,
            profiles: Vec::new(),
            profile: 1,
            active: None,
            section: Section::Buttons,
            device: None,
            draft: [0; 20],
            settings_device: None,
            settings: General([0; GENERAL_LEN]),
            selected: 0,
            category: Category::Mouse,
            capturing: false,
            just_captured: false,
            applying: false,
            busy: true,
            status: (Tone::Info, "A procurar o rato…".into()),
        }
    }

    fn buttons_dirty(&self) -> bool {
        self.device.is_some_and(|d| d != self.draft)
    }

    fn settings_dirty(&self) -> bool {
        self.settings_device.is_some_and(|d| d != self.settings)
    }

    fn dirty(&self) -> bool {
        self.buttons_dirty() || self.settings_dirty()
    }

    fn take_snapshot(&mut self, s: Snapshot) {
        self.profile = s.profile;
        self.device = Some(s.buttons);
        self.draft = s.buttons;
        self.settings_device = Some(s.settings);
        self.settings = s.settings;
        if s.active.is_some() {
            self.active = s.active;
        }
    }

    fn action(&self, index: usize) -> Action {
        Action::decode(self.draft[BUTTONS[index].slot - 1])
    }

    fn set_action(&mut self, action: Action) {
        self.draft[BUTTONS[self.selected].slot - 1] = action.encode();
    }

    fn select(&mut self, index: usize) {
        self.selected = index;
        self.category = Category::of(self.action(index));
        self.capturing = false;
    }

    fn load(&mut self, profile: u8) {
        self.profile = profile;
        self.busy = true;
        self.status = (Tone::Info, format!("A ler o perfil {profile}…"));
        self.send(Request::Load(profile));
    }

    fn send(&self, request: Request) {
        if let Some(w) = &self.worker {
            w.send(request);
        }
    }

    fn poll(&mut self) {
        while let Some(reply) = self.worker.as_ref().and_then(Worker::poll) {
            match reply {
                Reply::Connected { profiles, active } => {
                    self.connected = true;
                    self.active = active;
                    let first = active.filter(|a| profiles.contains(a)).or(profiles.first().copied()).unwrap_or(1);
                    self.profiles = profiles;
                    self.load(first);
                }
                Reply::Loaded(s) => {
                    self.busy = false;
                    let p = s.profile;
                    self.take_snapshot(s);
                    self.select(self.selected);
                    self.status = (Tone::Info, format!("Perfil {p} lido do rato."));
                }
                Reply::Applied(s) => {
                    self.busy = false;
                    self.applying = false;
                    let p = s.profile;
                    self.take_snapshot(s);
                    self.active = Some(p);
                    self.status = (Tone::Ok, format!("Perfil {p} gravado, confirmado e activo no rato."));
                }
                Reply::Activated(p) => {
                    self.busy = false;
                    self.active = Some(p);
                    self.status = (Tone::Ok, format!("O rato está agora a usar o perfil {p}."));
                }
                Reply::Failed(e) => {
                    self.busy = false;
                    if self.device.is_none() {
                        self.connected = false;
                    }
                    self.status = (Tone::Error, e);
                    if std::mem::take(&mut self.applying) {
                        self.busy = true;
                        self.send(Request::Load(self.profile));
                    }
                }
            }
        }
    }

    fn capture_key(&mut self, ui: &Ui) {
        if !self.capturing {
            return;
        }
        let pressed = ui.input(|i| {
            i.events.iter().find_map(|e| match e {
                egui::Event::Key { key, pressed: true, modifiers, .. } => keys::hid_usage(*key).map(|u| (keys::modifier_bits(*modifiers), u)),
                egui::Event::Copy => Some((MOD_CMD, 0x06)),
                egui::Event::Cut => Some((MOD_CMD, 0x1B)),
                egui::Event::Paste(_) => Some((MOD_CMD, 0x19)),
                _ => None,
            })
        });
        if let Some((mods, key)) = pressed {
            self.set_action(Action::Key { mods, key });
            self.capturing = false;
            self.just_captured = true;
        }
    }

    fn titlebar(&mut self, ui: &mut Ui) {
        let frame = Frame::new().fill(Tokens::TITLEBAR).inner_margin(Margin::symmetric(18, 0));
        egui::Panel::top("titlebar").exact_size(52.0).frame(frame).show_separator_line(false).show(ui, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter().clone();
            let mid = rect.center().y;
            let brand = painter.text(Pos2::new(rect.left(), mid), Align2::LEFT_CENTER, "MORFIX", theme::semibold(15.0), Tokens::TEXT);
            painter.text(
                Pos2::new(brand.right() + 8.0, mid),
                Align2::LEFT_CENTER,
                "|  GXT 970",
                theme::regular(15.0),
                Tokens::TEXT_MUTED,
            );

            let tab_w = 96.0;
            let total = tab_w * PROFILES.len() as f32;
            let mut x = rect.center().x - total / 2.0;
            for p in PROFILES {
                let tab_rect = Rect::from_min_size(Pos2::new(x, rect.top() + 10.0), vec2(tab_w, 32.0));
                x += tab_w;
                let available = self.profiles.contains(&p);
                let selected = p == self.profile && self.device.is_some();
                let enabled = available && !self.busy && !self.dirty();
                let label = format!("Perfil {p}");
                let response = ui
                    .add_enabled_ui(enabled || selected, |ui| {
                        Tab::new(Id::new(("profile", p)), &label, selected).show_at(ui, tab_rect, |ui, r| {
                            let fill = if selected {
                                Tokens::CARD
                            } else if r.hovered() && enabled {
                                Tokens::HOVER
                            } else {
                                Color32::TRANSPARENT
                            };
                            ui.painter().rect_filled(tab_rect.shrink2(vec2(3.0, 0.0)), CornerRadius::same(Tokens::RADIUS), fill);
                            if selected {
                                let bar = Rect::from_min_max(
                                    Pos2::new(tab_rect.left() + 18.0, tab_rect.bottom() - 2.0),
                                    Pos2::new(tab_rect.right() - 18.0, tab_rect.bottom()),
                                );
                                ui.painter().rect_filled(bar, CornerRadius::same(1), Tokens::BRAND);
                            }
                            let color = if !available {
                                Tokens::TEXT_FAINT.gamma_multiply(0.5)
                            } else if selected {
                                Tokens::TEXT
                            } else {
                                Tokens::TEXT_MUTED
                            };
                            ui.painter().text(tab_rect.center(), Align2::CENTER_CENTER, &label, theme::medium(13.0), color);
                            if self.active == Some(p) {
                                ui.painter().circle_filled(Pos2::new(tab_rect.right() - 14.0, tab_rect.center().y), 3.0, Tokens::OK);
                            }
                        })
                    })
                    .inner;
                let response = if self.active == Some(p) { response.on_hover_text("Perfil em uso no rato.") } else { response };
                let response = if !available {
                    response.on_disabled_hover_text("O rato não responde a este perfil.")
                } else if self.dirty() && !selected {
                    response.on_disabled_hover_text("Aplica ou descarta as alterações antes de mudar de perfil.")
                } else {
                    response
                };
                if response.clicked() && enabled && !selected {
                    self.load(p);
                }
            }

            let (dot, text) = if self.connected { (Tokens::OK, "Ligado") } else { (Color32::from_rgb(0xE0, 0x4F, 0x4F), "Desligado") };
            let right = Pos2::new(rect.right(), mid);
            let galley = painter.layout_no_wrap(text.into(), theme::regular(12.0), Tokens::TEXT_MUTED);
            let text_pos = Pos2::new(right.x - galley.size().x, mid - galley.size().y / 2.0);
            painter.circle_filled(Pos2::new(text_pos.x - 10.0, mid), 4.0, dot);
            painter.galley(text_pos, galley, Tokens::TEXT_MUTED);
            if !self.connected && !self.busy {
                let r = Rect::from_center_size(Pos2::new(text_pos.x - 70.0, mid), vec2(72.0, 26.0));
                if ui.put(r, egui::Button::new("Ligar")).clicked() {
                    self.busy = true;
                    self.status = (Tone::Info, "A procurar o rato…".into());
                    self.send(Request::Connect);
                }
            }
        });
    }

    fn footer(&mut self, ui: &mut Ui) {
        let frame = Frame::new().fill(Tokens::CHROME).inner_margin(Margin::symmetric(18, 10));
        egui::Panel::bottom("footer").exact_size(54.0).frame(frame).show(ui, |ui| {
            ui.horizontal_centered(|ui| {
                let (tone, text) = &self.status;
                let color = match tone {
                    Tone::Info => Tokens::TEXT_MUTED,
                    Tone::Ok => Tokens::OK,
                    Tone::Error => Color32::from_rgb(0xF2, 0x6D, 0x6D),
                };
                if self.busy {
                    ui.spinner();
                }
                ui.label(RichText::new(text).color(color));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let has_left = !self.buttons_dirty() || self.draft.contains(&Action::LEFT_CLICK);
                    let can_apply = self.dirty() && !self.busy && has_left;
                    let apply = egui::Button::new(RichText::new("Aplicar").font(theme::medium(13.0)).color(Color32::WHITE))
                        .fill(if can_apply { Tokens::ACCENT } else { Tokens::ACCENT_SOFT })
                        .min_size(vec2(96.0, 30.0));
                    let r = ui.add_enabled(can_apply, apply);
                    let r = if !has_left { r.on_disabled_hover_text("Pelo menos um botão tem de ficar com o clique esquerdo.") } else { r };
                    if r.clicked() {
                        self.busy = true;
                        self.applying = true;
                        self.capturing = false;
                        self.status = (Tone::Info, format!("A gravar o perfil {}…", self.profile));
                        let buttons = self.buttons_dirty().then_some(self.draft);
                        let settings = self.settings_dirty().then(|| Box::new(self.settings));
                        self.send(Request::Apply { profile: self.profile, buttons, settings });
                    }
                    if ui.add_enabled(self.dirty() && !self.busy, egui::Button::new("Descartar").min_size(vec2(0.0, 30.0))).clicked() {
                        if let Some(d) = self.device {
                            self.draft = d;
                        }
                        if let Some(g) = self.settings_device {
                            self.settings = g;
                        }
                        self.select(self.selected);
                    }
                    if self.active != Some(self.profile)
                        && self.device.is_some()
                        && ui
                            .add_enabled(!self.busy && !self.dirty(), egui::Button::new("Usar este perfil").min_size(vec2(0.0, 30.0)))
                            .on_hover_text("Muda o perfil em uso no rato para este.")
                            .on_disabled_hover_text("Aplica ou descarta as alterações primeiro.")
                            .clicked()
                    {
                        self.busy = true;
                        self.status = (Tone::Info, format!("A activar o perfil {}…", self.profile));
                        self.send(Request::Activate(self.profile));
                    }
                    let factory = factory_map();
                    let differs = self.draft != factory;
                    if self.section == Section::Buttons
                        && ui
                        .add_enabled(differs && !self.busy && self.device.is_some(), egui::Button::new("Repor de fábrica").min_size(vec2(0.0, 30.0)))
                        .on_hover_text("Prepara as funções de origem; só são gravadas ao carregar em Aplicar.")
                        .clicked()
                    {
                        self.draft = factory;
                        self.select(self.selected);
                    }
                });
            });
        });
    }

    fn side_panel(&mut self, ui: &mut Ui) {
        let frame = Frame::new().fill(Tokens::CHROME).inner_margin(Margin::symmetric(16, 16));
        egui::Panel::right("assign").exact_size(372.0).resizable(false).frame(frame).show(ui, |ui| {
            self.section_bar(ui);
            ui.add_space(12.0);
            ui.add_enabled_ui(self.device.is_some() && !self.busy, |ui| match self.section {
                Section::Buttons => {
                    ui.label(RichText::new("Atribuição de botões").font(theme::semibold(16.0)));
                    ui.label(RichText::new("Escolhe um botão na imagem ou na lista.").color(Tokens::TEXT_FAINT).size(12.0));
                    ui.add_space(8.0);
                    self.button_list(ui);
                    ui.add_space(10.0);
                    self.editor(ui);
                }
                section => {
                    egui::ScrollArea::vertical().id_salt(("settings", section.label())).auto_shrink([false, false]).show(ui, |ui| match section {
                        Section::Dpi => crate::settings::dpi(self, ui),
                        Section::Lighting => crate::settings::lighting(self, ui),
                        _ => crate::settings::performance(self, ui),
                    });
                }
            });
        });
    }

    fn section_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            ui.spacing_mut().button_padding.x = 2.0;
            let w = (ui.available_width() - 12.0) / Section::ALL.len() as f32;
            for s in Section::ALL {
                let on = self.section == s;
                let changed = match s {
                    Section::Buttons => self.buttons_dirty(),
                    _ => crate::settings::section_dirty(self, s),
                };
                let mut text = RichText::new(s.label()).font(theme::medium(12.0)).color(if on { Tokens::TEXT } else { Tokens::TEXT_MUTED });
                if changed {
                    text = text.color(Tokens::ACCENT_TEXT);
                }
                let b = egui::Button::new(text)
                    .selected(on)
                    .fill(if on { Tokens::CARD } else { Tokens::FIELD })
                    .stroke(Stroke::new(1.0, if on { Tokens::BORDER } else { Tokens::DIVIDER }));
                if ui.add_sized(vec2(w, 30.0), b).clicked() {
                    self.section = s;
                    self.capturing = false;
                }
            }
        });
    }

    fn button_list(&mut self, ui: &mut Ui) {
        let row_h = 30.0;
        let height = (ui.available_height() - 250.0).clamp(row_h * 5.5, row_h * BUTTONS.len() as f32);
        egui::ScrollArea::vertical().id_salt("list").max_height(height).show(ui, |ui| {
            for (i, b) in BUTTONS.iter().enumerate() {
                let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), row_h), Sense::click());
                let selected = i == self.selected;
                let fill = if selected {
                    Tokens::SELECTED
                } else if response.hovered() {
                    Tokens::HOVER
                } else {
                    Color32::TRANSPARENT
                };
                let painter = ui.painter();
                painter.rect_filled(rect, CornerRadius::same(Tokens::RADIUS), fill);
                let badge = Pos2::new(rect.left() + 18.0, rect.center().y);
                painter.circle(badge, 11.0, Tokens::FIELD, Stroke::new(1.0, if selected { Tokens::ACCENT } else { Tokens::BORDER }));
                painter.text(badge, Align2::CENTER_CENTER, b.badge, theme::medium(10.5), Tokens::TEXT);
                painter.text(Pos2::new(rect.left() + 38.0, rect.center().y), Align2::LEFT_CENTER, b.name, theme::regular(13.0), Tokens::TEXT);
                let action = self.action(i);
                let changed = self.device.is_some_and(|d| d[b.slot - 1] != self.draft[b.slot - 1]);
                let right = painter.text(
                    Pos2::new(rect.right() - 10.0, rect.center().y),
                    Align2::RIGHT_CENTER,
                    action.label(),
                    theme::regular(12.0),
                    if changed { Tokens::ACCENT_TEXT } else { Tokens::TEXT_MUTED },
                );
                if changed {
                    painter.circle_filled(Pos2::new(right.left() - 8.0, rect.center().y), 3.0, Tokens::ACCENT);
                }
                if response.clicked() {
                    self.select(i);
                }
            }
        });
    }

    fn editor(&mut self, ui: &mut Ui) {
        let b = &BUTTONS[self.selected];
        let current = self.action(self.selected);
        Frame::new().fill(Tokens::CARD).corner_radius(CornerRadius::same(8)).inner_margin(Margin::same(12)).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(b.name).font(theme::semibold(14.0)));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(current.label()).color(Tokens::ACCENT_TEXT));
                });
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                for c in Category::ALL {
                    let on = self.category == c;
                    let text = RichText::new(c.label()).size(12.0).color(if on { Tokens::TEXT } else { Tokens::TEXT_MUTED });
                    if ui.add(egui::Button::new(text).selected(on).min_size(vec2(0.0, 26.0))).clicked() {
                        self.category = c;
                        self.capturing = false;
                    }
                }
            });
            ui.add_space(8.0);
            egui::ScrollArea::vertical().id_salt("options").auto_shrink([false, true]).show(ui, |ui| match self.category {
                Category::Mouse => self.options(ui, MouseFn::ALL.iter().map(|&f| Action::Mouse(f)).collect(), current),
                Category::Dpi => self.options(ui, DpiFn::ALL.iter().map(|&f| Action::Dpi(f)).collect(), current),
                Category::Media => self.options(ui, MEDIA.iter().map(|&m| Action::Media(m)).collect(), current),
                Category::Keyboard => self.keyboard(ui, current),
                Category::Other => self.other(ui, current),
            });
        });
    }

    fn options(&mut self, ui: &mut Ui, actions: Vec<Action>, current: Action) {
        let w = ((ui.available_width() - 6.0) / 2.0).floor();
        egui::Grid::new("opts").num_columns(2).spacing(vec2(6.0, 6.0)).show(ui, |ui| {
            for (i, a) in actions.into_iter().enumerate() {
                let on = a == current;
                let option = egui::Button::new(a.label())
                    .selected(on)
                    .fill(if on { Tokens::SELECTED } else { Tokens::FIELD })
                    .stroke(Stroke::new(1.0, if on { Tokens::ACCENT } else { Tokens::DIVIDER }));
                if ui.add(option.min_size(vec2(w, 28.0))).clicked() {
                    self.set_action(a);
                }
                if i % 2 == 1 {
                    ui.end_row();
                }
            }
        });
    }

    fn keyboard(&mut self, ui: &mut Ui, current: Action) {
        let (mut mods, key) = match current {
            Action::Key { mods, key } => (mods, Some(key)),
            _ => (0, None),
        };
        let text = if self.capturing {
            "Carrega numa tecla…".to_owned()
        } else {
            key.map_or("Nenhuma tecla definida".to_owned(), |k| gxt970::buttons::key_combo_label(mods, k))
        };
        let field = egui::Button::new(RichText::new(text).font(theme::medium(14.0)))
            .selected(self.capturing)
            .min_size(vec2(ui.available_width(), 40.0));
        let response = ui.add(field).on_hover_text("Clica e carrega na tecla ou combinação. Atalhos do sistema como Cmd+Q não chegam aqui: escolhe a tecla e liga o Cmd em baixo.");
        if response.clicked() && !std::mem::take(&mut self.just_captured) {
            self.capturing = !self.capturing;
        }
        self.just_captured = false;
        ui.add_space(6.0);
        ui.add_enabled_ui(key.is_some() && !self.capturing, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                let w = (ui.available_width() - 18.0) / 4.0;
                for (bit, name) in [(MOD_CTRL, "Ctrl"), (MOD_ALT, "Alt"), (MOD_SHIFT, "Shift"), (MOD_CMD, "Cmd")] {
                    let on = mods & bit != 0;
                    let chip = egui::Button::new(name).selected(on).stroke(Stroke::new(1.0, if on { Tokens::ACCENT } else { Tokens::BORDER }));
                    if ui.add(chip.min_size(vec2(w, 28.0))).clicked() {
                        mods ^= bit;
                        if let Some(k) = key {
                            self.set_action(Action::Key { mods, key: k });
                        }
                    }
                }
            });
        });
    }

    fn other(&mut self, ui: &mut Ui, current: Action) {
        if ui.add(egui::Button::new("Desactivar botão").selected(current == Action::Disabled).min_size(vec2(ui.available_width(), 28.0))).clicked() {
            self.set_action(Action::Disabled);
        }
        if let Action::Other(_) | Action::Empty = current {
            ui.add_space(6.0);
            ui.label(
                RichText::new("A função actual vem de outra configuração e fica como está enquanto não escolheres outra.")
                    .color(Tokens::TEXT_FAINT)
                    .size(12.0),
            );
        }
    }

    fn stage(&mut self, ui: &mut Ui) {
        egui::CentralPanel::default().frame(Frame::new().fill(Tokens::PASTEBOARD)).show(ui, |ui| {
            let rect = ui.max_rect();
            let painter = ui.painter().clone();
            let glow = rect.center() - vec2(0.0, rect.height() * 0.12);
            let tint = crate::settings::glow_tint(self);
            for i in 0..24 {
                let t = i as f32 / 24.0;
                let r = rect.width().min(rect.height()) * (0.75 - t * 0.6);
                let a = (3.0 + t * 2.0) as u8;
                let c = tint.map_or(Color32::from_white_alpha(a), |[r, g, b]| Color32::from_rgba_unmultiplied(r, g, b, a * 2));
                painter.circle_filled(glow, r, c);
            }
            let floor = rect.top() + rect.height() * 0.80;
            painter.rect_filled(Rect::from_min_max(Pos2::new(rect.left(), floor), rect.max), CornerRadius::ZERO, Color32::from_black_alpha(70));
            painter.line_segment([Pos2::new(rect.left(), floor), Pos2::new(rect.right(), floor)], Stroke::new(1.0, Color32::from_white_alpha(14)));

            let scale = ((rect.height() - 90.0) / MOUSE_SIZE.y).min((rect.width() - 40.0) / MOUSE_SIZE.x).clamp(0.6, 1.3);
            let size = MOUSE_SIZE * scale;
            let origin = Pos2::new(rect.center().x - size.x / 2.0, rect.top() + (rect.height() - size.y) / 2.0 - 10.0);
            for (x, w) in [(113.0, 100.0), (318.0, 70.0)] {
                let center = Pos2::new(origin.x + x * scale, origin.y + size.y + 4.0);
                for i in 0..6 {
                    let k = 1.0 - i as f32 * 0.12;
                    painter.add(egui::epaint::EllipseShape::filled(center, vec2(w * scale * k, 7.0 * scale * k), Color32::from_black_alpha(28)));
                }
            }
            let texture = self
                .texture
                .get_or_insert_with(|| ui.ctx().load_texture("mouse", load_png(MOUSE_PNG), egui::TextureOptions::LINEAR));
            painter.image(texture.id(), Rect::from_min_size(origin, size), Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);

            if self.section != Section::Buttons {
                crate::settings::stage_overlay(self, ui, rect);
                return;
            }
            let mut clicked = None;
            for (i, b) in BUTTONS.iter().enumerate() {
                let center = origin + vec2(b.pos.0, b.pos.1) * scale;
                let radius = BADGE_RADIUS * scale;
                let hit = Rect::from_center_size(center, Vec2::splat(radius * 2.4));
                let response = ui.interact(hit, Id::new(("badge", i)), Sense::click());
                let selected = i == self.selected || self.show_all_rings;
                let changed = self.device.is_some_and(|d| d[b.slot - 1] != self.draft[b.slot - 1]);
                if selected {
                    ui.painter().circle(center, radius + 3.0, Tokens::ACCENT.gamma_multiply(0.25), Stroke::new(2.0, Tokens::ACCENT));
                } else if response.hovered() {
                    ui.painter().circle_stroke(center, radius + 2.0, Stroke::new(1.5, Tokens::TEXT_MUTED));
                } else if changed {
                    ui.painter().circle_stroke(center, radius + 2.0, Stroke::new(1.5, Tokens::ACCENT_TEXT));
                }
                let response = response.on_hover_text(format!("{} · {}", b.name, self.action(i).label()));
                if response.clicked() {
                    clicked = Some(i);
                }
            }
            if let Some(i) = clicked
                && self.device.is_some()
            {
                self.select(i);
            }

            let caption = format!("{}  →  {}", BUTTONS[self.selected].name, self.action(self.selected).label());
            ui.painter().text(
                Pos2::new(rect.center().x, rect.bottom() - 24.0),
                Align2::CENTER_CENTER,
                caption,
                theme::medium(14.0),
                Tokens::TEXT,
            );
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.draw(ui);
    }
}

impl App {
    fn draw(&mut self, ui: &mut Ui) {
        if !self.themed {
            theme::install(ui.ctx());
            self.themed = true;
            ui.ctx().request_repaint();
            return;
        }
        self.poll();
        self.capture_key(ui);
        self.titlebar(ui);
        self.footer(ui);
        self.side_panel(ui);
        self.stage(ui);
    }
}

fn load_png(bytes: &[u8]) -> egui::ColorImage {
    match image::load_from_memory(bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            egui::ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], rgba.as_raw())
        }
        Err(_) => egui::ColorImage::new([1, 1], vec![Color32::TRANSPARENT]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui_kittest::Harness;

    fn preview(map: ButtonMap, profiles: Vec<u8>) -> App {
        let mut app = App::with_worker(None);
        app.connected = true;
        app.busy = false;
        app.profiles = profiles;
        app.device = Some(map);
        app.draft = map;
        let backup = include_bytes!("../../../fixtures/general-backup.bin");
        let g = General::from_slice(&backup[gxt970::protocol::BLOCK_OFFSET..]);
        app.settings_device = Some(g);
        app.settings = g;
        app.active = Some(1);
        app.status = (Tone::Info, "Perfil 1 lido do rato.".into());
        app
    }

    #[test]
    fn renders_preview() {
        let Ok(out) = std::env::var("GXT970_PREVIEW") else { return };
        let mut app = preview(factory_map(), vec![1, 2]);
        app.draft[BUTTONS[5].slot - 1] = Action::Key { mods: MOD_CMD, key: 0x06 }.encode();
        let variant = std::env::var("GXT970_VARIANT").unwrap_or_default();
        app.show_all_rings = variant == "all";
        app.section = match variant.as_str() {
            "dpi" => Section::Dpi,
            "led" => Section::Lighting,
            "perf" => Section::Performance,
            _ => Section::Buttons,
        };
        if variant == "led" {
            app.settings.set_effect(3);
        }
        if variant == "mouse" {
            app.selected = 3;
            app.category = Category::Mouse;
        } else {
            app.selected = 5;
            app.category = Category::Keyboard;
        }
        let mut harness = Harness::builder().with_size(vec2(1030.0, 662.0)).wgpu().build_ui_state(|ui, app: &mut App| app.draw(ui), app);
        harness.run();
        match harness.render() {
            Ok(img) => img.save(&out).unwrap_or_else(|e| eprintln!("{e}")),
            Err(e) => eprintln!("{e}"),
        }
    }
}
