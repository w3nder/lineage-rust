use crate::lobby::config::ServerConfig;
use egui::{
    Align, Align2, Area, Button, Color32, ComboBox, Context, CornerRadius, Frame, Id, Key, Label, Layout, Margin,
    Order, RichText, Sense, Stroke, TextEdit, Ui, Vec2, Window, pos2, vec2,
};

pub const PANEL_FILL: Color32 = Color32::from_rgba_premultiplied(14, 11, 8, 224);
pub const PANEL_STROKE: Color32 = Color32::from_rgb(98, 80, 42);
pub const FIELD_FILL: Color32 = Color32::from_rgb(9, 8, 6);
pub const FIELD_STROKE: Color32 = Color32::from_rgb(58, 48, 28);
pub const GOLD: Color32 = Color32::from_rgb(214, 190, 126);
pub const GOLD_DIM: Color32 = Color32::from_rgb(160, 138, 86);
pub const BUTTON_FILL: Color32 = Color32::from_rgb(62, 48, 22);
pub const BUTTON_STROKE: Color32 = Color32::from_rgb(178, 148, 74);
pub const INK: Color32 = Color32::from_rgb(232, 222, 196);
pub const MUTED: Color32 = Color32::from_rgb(150, 140, 118);
pub const HINT: Color32 = Color32::from_rgb(96, 90, 80);
pub const ERROR: Color32 = Color32::from_rgb(220, 120, 100);
pub const SLOT_FILL: Color32 = Color32::from_rgba_premultiplied(20, 16, 10, 210);
pub const SLOT_SELECTED: Color32 = Color32::from_rgb(214, 190, 126);
pub const CAPS_SIZE: f32 = 14.0;
pub const CAPS_SMALL: f32 = 11.5;
pub const LOGIN_WIDTH: f32 = 380.0;
pub const BUTTON_SIZE: Vec2 = vec2(124.0, 26.0);
pub const WIDE_BUTTON: Vec2 = vec2(332.0, 26.0);
pub const SLOT_SIZE: Vec2 = vec2(100.0, 38.0);
pub const LOGIN_OFFSET_Y: f32 = 0.0;
pub const RACES: [&str; 5] = ["Human", "Elf", "Dark Elf", "Orc", "Dwarf"];
pub const CLASSES: [&str; 2] = ["Fighter", "Mystic"];
pub const SEXES: [&str; 2] = ["Male", "Female"];
pub const HAIR_STYLES: i32 = 7;
pub const HAIR_COLORS: i32 = 4;
pub const FACES: i32 = 3;

fn frame() -> Frame {
    Frame::new()
        .fill(PANEL_FILL)
        .stroke(Stroke::new(1.0, PANEL_STROKE))
        .corner_radius(CornerRadius::same(2))
        .inner_margin(Margin::same(18))
}

fn panel<'a>(id: &'static str, anchor: Align2, offset: Vec2) -> Window<'a> {
    Window::new(id)
        .id(Id::new(id))
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(anchor, offset)
        .frame(frame())
}

pub fn small_caps(ui: &mut Ui, text: &str, size: f32, color: Color32) {
    ui.add(Label::new(caps_text(text, size, color)).selectable(false));
}

pub fn caps_text(text: &str, size: f32, color: Color32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let font = |points: f32| egui::FontId::new(points, egui::FontFamily::Proportional);
    for (index, word) in text.split(' ').enumerate() {
        if index > 0 {
            job.append(" ", 0.0, egui::TextFormat { font_id: font(size), color, ..Default::default() });
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            job.append(
                &first.to_uppercase().to_string(),
                0.0,
                egui::TextFormat { font_id: font(size), color, ..Default::default() },
            );
        }
        let rest: String = chars.flat_map(char::to_uppercase).collect();
        if !rest.is_empty() {
            job.append(
                &rest,
                0.0,
                egui::TextFormat {
                    font_id: font(size * CAPS_SMALL / CAPS_SIZE),
                    color,
                    extra_letter_spacing: 0.6,
                    ..Default::default()
                },
            );
        }
    }
    job
}

pub fn gold_button(ui: &mut Ui, text: &str, size: Vec2, enabled: bool) -> bool {
    let fill = if enabled { BUTTON_FILL } else { Color32::from_rgb(34, 28, 16) };
    let stroke = if enabled { BUTTON_STROKE } else { FIELD_STROKE };
    let color = if enabled { INK } else { HINT };
    let button = Button::new(caps_text(text, 13.0, color))
        .fill(fill)
        .stroke(Stroke::new(1.0, stroke))
        .corner_radius(CornerRadius::same(2))
        .min_size(size);
    ui.add_enabled(enabled, button).clicked()
}

fn field(ui: &mut Ui, value: &mut String, hint: &str, password: bool, width: f32) -> egui::Response {
    let edit = TextEdit::singleline(value)
        .hint_text(RichText::new(hint).color(HINT))
        .password(password)
        .desired_width(width)
        .text_color(INK)
        .background_color(FIELD_FILL)
        .margin(Margin::symmetric(8, 5));
    ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0, FIELD_STROKE);
    ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::new(1.0, GOLD_DIM);
    ui.visuals_mut().widgets.active.bg_stroke = Stroke::new(1.0, GOLD);
    ui.visuals_mut().selection.stroke = Stroke::new(1.0, GOLD);
    ui.add(edit)
}

fn status_line(ui: &mut Ui, status: &str) {
    if !status.is_empty() {
        ui.add_space(8.0);
        ui.add(Label::new(RichText::new(status).size(12.0).color(ERROR)).wrap());
    }
}

pub fn version_tag(ctx: &Context, version: &str) {
    Area::new(Id::new("versão"))
        .order(Order::Background)
        .anchor(Align2::RIGHT_BOTTOM, vec2(-10.0, -6.0))
        .interactable(false)
        .show(ctx, |ui| {
            ui.label(RichText::new(format!("v{version}")).size(11.0).color(MUTED));
        });
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginAction {
    Connect,
    Exit,
}

pub struct LoginForm {
    pub account: String,
    pub password: String,
    pub status: String,
    pub connecting: bool,
    pub settings_open: bool,
    pub host: String,
    pub port: String,
    pub saved: Option<Result<(), String>>,
}

impl LoginForm {
    pub fn new(account: String, password: String, config: &ServerConfig) -> Self {
        Self {
            account,
            password,
            status: String::new(),
            connecting: false,
            settings_open: false,
            host: config.host.clone(),
            port: config.port.to_string(),
            saved: None,
        }
    }

    pub fn from_env(config: &ServerConfig) -> Self {
        Self::new(
            std::env::var("KILLER_ACCOUNT").unwrap_or_default(),
            std::env::var("KILLER_PASSWORD").unwrap_or_default(),
            config,
        )
    }

    pub fn ready(&self) -> bool {
        !self.connecting && !self.account.is_empty() && !self.password.is_empty() && self.config().is_some()
    }

    pub fn config(&self) -> Option<ServerConfig> {
        let host = self.host.trim();
        let port = self.port.trim().parse().ok()?;
        (!host.is_empty()).then(|| ServerConfig { host: host.to_string(), port })
    }
}

pub fn login(ctx: &Context, form: &mut LoginForm) -> Option<LoginAction> {
    let mut action = None;
    panel("login", Align2::CENTER_CENTER, vec2(0.0, LOGIN_OFFSET_Y)).show(ctx, |ui| {
        ui.set_width(LOGIN_WIDTH - 36.0);
        let mut submitted = false;
        ui.add_enabled_ui(!form.connecting, |ui| {
            egui::Grid::new("login-fields").num_columns(2).spacing([10.0, 10.0]).show(ui, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| small_caps(ui, "Account", CAPS_SIZE, GOLD));
                let account = field(ui, &mut form.account, "Account name", false, 240.0);
                submitted |= account.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                ui.end_row();
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| small_caps(ui, "Password", CAPS_SIZE, GOLD));
                let password = field(ui, &mut form.password, "Password", true, 240.0);
                submitted |= password.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
                ui.end_row();
            });
            ui.add_space(22.0);
            ui.horizontal(|ui| {
                ui.add_space(96.0);
                if gold_button(ui, "Log In", BUTTON_SIZE, !form.connecting) {
                    if form.ready() {
                        action = Some(LoginAction::Connect);
                    } else {
                        form.status = "informe a conta e a senha".to_string();
                    }
                }
                ui.add_space(10.0);
                if gold_button(ui, "Exit", BUTTON_SIZE, true) {
                    action = Some(LoginAction::Exit);
                }
            });
        });
        if submitted && form.ready() {
            action = Some(LoginAction::Connect);
        }
        status_line(ui, &form.status);
    });
    #[cfg(not(target_arch = "wasm32"))]
    settings_gear(ctx, form);
    action
}

#[cfg(not(target_arch = "wasm32"))]
fn settings_gear(ctx: &Context, form: &mut LoginForm) {
    Area::new(Id::new("engrenagem")).order(Order::Foreground).anchor(Align2::LEFT_BOTTOM, vec2(10.0, -8.0)).show(
        ctx,
        |ui| {
            let gear = Button::new(RichText::new("⚙").size(18.0).color(if form.settings_open { GOLD } else { MUTED }))
                .frame(false);
            if ui.add(gear).clicked() {
                form.settings_open = !form.settings_open;
            }
        },
    );
    if !form.settings_open {
        return;
    }
    panel("servidor", Align2::LEFT_BOTTOM, vec2(10.0, -40.0)).show(ctx, |ui| {
        ui.set_width(260.0);
        small_caps(ui, "Login Server", CAPS_SIZE, GOLD);
        ui.add_space(8.0);
        egui::Grid::new("servidor-campos").num_columns(2).spacing([10.0, 8.0]).show(ui, |ui| {
            ui.label(RichText::new("Host").size(12.0).color(MUTED));
            field(ui, &mut form.host, "server.l2killer.com", false, 180.0);
            ui.end_row();
            ui.label(RichText::new("Porta").size(12.0).color(MUTED));
            field(ui, &mut form.port, "2106", false, 80.0);
            ui.end_row();
        });
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let valid = form.config().is_some();
            if gold_button(ui, "Save", vec2(96.0, 24.0), valid)
                && let Some(config) = form.config()
            {
                form.saved = Some(config.save().map_err(|e| e.to_string()));
            }
            ui.add_space(8.0);
            if gold_button(ui, "Close", vec2(96.0, 24.0), true) {
                form.settings_open = false;
            }
        });
        match &form.saved {
            Some(Ok(())) => {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!(
                        "salvo em {}",
                        ServerConfig::path().map(|p| p.display().to_string()).unwrap_or_default()
                    ))
                    .size(11.0)
                    .color(MUTED),
                );
            }
            Some(Err(e)) => {
                ui.add_space(6.0);
                ui.label(RichText::new(format!("não salvou: {e}")).size(11.0).color(ERROR));
            }
            None => {}
        }
    });
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerRow {
    pub id: u8,
    pub name: String,
    pub players: u16,
    pub max_players: u16,
    pub online: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServersAction {
    Choose(u8),
    Back,
}

pub fn servers(ctx: &Context, rows: &[ServerRow], status: &str) -> Option<ServersAction> {
    let mut action = None;
    panel("servidores", Align2::CENTER_CENTER, vec2(0.0, LOGIN_OFFSET_Y)).show(ctx, |ui| {
        ui.set_width(LOGIN_WIDTH - 36.0);
        ui.vertical_centered(|ui| small_caps(ui, "Select Server", 15.0, GOLD));
        ui.add_space(14.0);
        for row in rows {
            let (rect, response) = ui.allocate_exact_size(vec2(WIDE_BUTTON.x, 48.0), Sense::click());
            let hovered = response.hovered();
            let fill = if hovered { Color32::from_rgb(28, 22, 12) } else { FIELD_FILL };
            ui.painter().rect(
                rect,
                CornerRadius::same(2),
                fill,
                Stroke::new(1.0, if hovered { GOLD_DIM } else { FIELD_STROKE }),
                egui::StrokeKind::Inside,
            );
            let name = caps_text(&row.name, 13.0, if row.online { INK } else { MUTED });
            let galley = ui.painter().layout_job(name);
            ui.painter().with_clip_rect(rect.shrink(8.0)).galley(pos2(rect.left() + 10.0, rect.top() + 6.0), galley, INK);
            ui.painter().text(
                pos2(rect.left() + 10.0, rect.bottom() - 12.0),
                Align2::LEFT_CENTER,
                if row.online { "Online" } else { "Offline" },
                egui::FontId::proportional(12.0),
                if row.online { GOLD } else { MUTED },
            );
            let count = format!("Jogadores: {} / {}", row.players, row.max_players);
            ui.painter().text(
                pos2(rect.right() - 10.0, rect.bottom() - 12.0),
                Align2::RIGHT_CENTER,
                count,
                egui::FontId::proportional(12.0),
                MUTED,
            );
            if response.clicked() {
                action = Some(ServersAction::Choose(row.id));
            }
            ui.add_space(4.0);
        }
        if rows.is_empty() {
            ui.label(RichText::new("aguardando a lista de servidores…").size(12.0).color(MUTED));
        }
        ui.add_space(14.0);
        if gold_button(ui, "Back", WIDE_BUTTON, true) {
            action = Some(ServersAction::Back);
        }
        status_line(ui, status);
    });
    action
}

#[derive(Debug, Clone, PartialEq)]
pub struct CharacterSlot {
    pub name: String,
    pub class_name: String,
    pub level: i32,
    pub deleting: bool,
    pub label: Option<[f32; 2]>,
    pub delete_label: Option<[f32; 2]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharactersAction {
    Select(usize),
    Enter,
    Back,
    Delete,
    Create,
}

pub fn characters(
    ctx: &Context,
    slots: &[CharacterSlot],
    selected: Option<usize>,
    status: &str,
    busy: bool,
) -> Option<CharactersAction> {
    let mut action = None;
    for (index, slot) in slots.iter().enumerate() {
        if let Some([x, y]) = slot.label {
            Area::new(Id::new(("rótulo", index)))
                .order(Order::Background)
                .fixed_pos(pos2(x, y))
                .pivot(Align2::CENTER_BOTTOM)
                .interactable(false)
                .show(ctx, |ui| {
                    ui.label(RichText::new(&slot.name).size(12.0).color(INK));
                });
        }
    }
    if let Some(slot) = selected.and_then(|i| slots.get(i)) {
        panel("cartão", Align2::CENTER_CENTER, vec2(190.0, -110.0)).show(ctx, |ui| {
            ui.set_width(130.0);
            small_caps(ui, &slot.name, CAPS_SIZE, GOLD);
            ui.add_space(4.0);
            ui.label(RichText::new(&slot.class_name).size(12.0).color(MUTED));
            ui.label(RichText::new(format!("Level {}", slot.level)).size(12.0).color(MUTED));
            if slot.deleting {
                ui.label(RichText::new("exclusão pendente").size(11.0).color(ERROR));
            }
        });
    }
    Area::new(Id::new("entrar")).order(Order::Foreground).anchor(Align2::CENTER_BOTTOM, vec2(0.0, -76.0)).show(
        ctx,
        |ui| {
            if gold_button(ui, "Enter World", vec2(160.0, 28.0), selected.is_some() && !busy) {
                action = Some(CharactersAction::Enter);
            }
        },
    );
    Area::new(Id::new("slots")).order(Order::Foreground).anchor(Align2::CENTER_BOTTOM, vec2(0.0, -20.0)).show(
        ctx,
        |ui| {
            Frame::new().fill(SLOT_FILL).inner_margin(Margin::symmetric(8, 6)).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    for (index, slot) in slots.iter().enumerate() {
                        let chosen = selected == Some(index);
                        let (rect, response) = ui.allocate_exact_size(SLOT_SIZE, Sense::click());
                        let stroke =
                            if chosen { Stroke::new(1.5, SLOT_SELECTED) } else { Stroke::new(1.0, FIELD_STROKE) };
                        ui.painter().rect(rect, CornerRadius::same(2), FIELD_FILL, stroke, egui::StrokeKind::Inside);
                        let galley =
                            ui.painter().layout_job(caps_text(&slot.name, 12.0, if chosen { GOLD } else { INK }));
                        ui.painter().galley(pos2(rect.left() + 8.0, rect.top() + 5.0), galley, INK);
                        ui.painter().text(
                            pos2(rect.left() + 8.0, rect.bottom() - 6.0),
                            Align2::LEFT_BOTTOM,
                            format!("Level {}", slot.level),
                            egui::FontId::proportional(10.0),
                            MUTED,
                        );
                        if response.clicked() && !busy {
                            action = Some(CharactersAction::Select(index));
                        }
                    }
                    if slots.len() < 7 {
                        let (rect, response) = ui.allocate_exact_size(SLOT_SIZE, Sense::click());
                        ui.painter().rect(
                            rect,
                            CornerRadius::same(2),
                            FIELD_FILL,
                            Stroke::new(1.0, FIELD_STROKE),
                            egui::StrokeKind::Inside,
                        );
                        ui.painter().text(
                            pos2(rect.left() + 8.0, rect.top() + 5.0),
                            Align2::LEFT_TOP,
                            "+",
                            egui::FontId::proportional(13.0),
                            INK,
                        );
                        ui.painter().text(
                            pos2(rect.left() + 8.0, rect.bottom() - 6.0),
                            Align2::LEFT_BOTTOM,
                            "Create",
                            egui::FontId::proportional(10.0),
                            MUTED,
                        );
                        if response.clicked() && !busy {
                            action = Some(CharactersAction::Create);
                        }
                    }
                });
            });
        },
    );
    Area::new(Id::new("voltar")).order(Order::Foreground).anchor(Align2::LEFT_BOTTOM, vec2(16.0, -20.0)).show(
        ctx,
        |ui| {
            if gold_button(ui, "Back", vec2(72.0, 22.0), !busy) {
                action = Some(CharactersAction::Back);
            }
        },
    );
    Area::new(Id::new("excluir")).order(Order::Foreground).anchor(Align2::RIGHT_BOTTOM, vec2(-16.0, -20.0)).show(
        ctx,
        |ui| {
            if gold_button(ui, "Delete", vec2(72.0, 22.0), selected.is_some() && !busy) {
                action = Some(CharactersAction::Delete);
            }
        },
    );
    if !status.is_empty() {
        Area::new(Id::new("estado"))
            .order(Order::Foreground)
            .anchor(Align2::CENTER_TOP, vec2(0.0, 16.0))
            .interactable(false)
            .show(ctx, |ui| {
                Frame::new().fill(PANEL_FILL).inner_margin(Margin::symmetric(12, 6)).show(ui, |ui| {
                    ui.label(RichText::new(status).size(12.0).color(INK));
                });
            });
    }
    action
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CreateForm {
    pub name: String,
    pub race: Option<usize>,
    pub class: Option<usize>,
    pub female: Option<bool>,
    pub hair_style: i32,
    pub hair_color: i32,
    pub face: i32,
    pub status: String,
    pub sending: bool,
    pub zoomed: bool,
}

impl CreateForm {
    pub fn class_id(&self) -> Option<i32> {
        let mage = self.class? == 1;
        Some(match (self.race?, mage) {
            (0, false) => 0,
            (0, true) => 10,
            (1, false) => 18,
            (1, true) => 25,
            (2, false) => 31,
            (2, true) => 38,
            (3, false) => 44,
            (3, true) => 49,
            (4, false) => 53,
            _ => return None,
        })
    }

    pub fn complete(&self) -> bool {
        !self.name.trim().is_empty() && self.female.is_some() && self.class_id().is_some()
    }

    pub fn class_info_id(&self) -> u32 {
        match self.class_id() {
            Some(0) => 1,
            Some(10) => 2,
            Some(18) => 3,
            Some(25) => 4,
            Some(31) => 5,
            Some(38) => 6,
            Some(44) => 7,
            Some(49) => 8,
            Some(53) => 9,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateAction {
    Changed,
    Look,
    Zoom,
    Create,
    Cancel,
}

fn arrows(ui: &mut Ui, label: &str, value: &mut i32, count: i32) -> bool {
    let mut changed = false;
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| small_caps(ui, label, 12.5, GOLD));
    ui.horizontal(|ui| {
        if gold_button(ui, "‹", vec2(22.0, 20.0), true) {
            *value = (*value - 1).rem_euclid(count);
            changed = true;
        }
        let (rect, _) = ui.allocate_exact_size(vec2(60.0, 20.0), Sense::hover());
        ui.painter().rect(
            rect,
            CornerRadius::same(2),
            FIELD_FILL,
            Stroke::new(1.0, FIELD_STROKE),
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            format!("Type {}", (b'A' + *value as u8) as char),
            egui::FontId::proportional(11.0),
            INK,
        );
        if gold_button(ui, "›", vec2(22.0, 20.0), true) {
            *value = (*value + 1).rem_euclid(count);
            changed = true;
        }
    });
    ui.end_row();
    changed
}

fn combo(ui: &mut Ui, id: &str, label: &str, choice: &mut Option<usize>, options: &[&str], hint: &str) -> bool {
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| small_caps(ui, label, 12.5, GOLD));
    let before = *choice;
    ui.visuals_mut().widgets.inactive.weak_bg_fill = FIELD_FILL;
    ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::new(1.0, FIELD_STROKE);
    ui.visuals_mut().widgets.hovered.weak_bg_fill = Color32::from_rgb(28, 22, 12);
    ComboBox::from_id_salt(id)
        .width(108.0)
        .selected_text(RichText::new(choice.map_or(hint, |i| options[i])).size(11.0).color(INK))
        .show_ui(ui, |ui| {
            for (index, option) in options.iter().enumerate() {
                ui.selectable_value(choice, Some(index), *option);
            }
        });
    ui.end_row();
    before != *choice
}

pub fn create(ctx: &Context, form: &mut CreateForm, description: &str) -> Option<CreateAction> {
    let mut action = None;
    panel("criação", Align2::LEFT_TOP, vec2(16.0, 16.0)).show(ctx, |ui| {
        ui.set_width(200.0);
        ui.vertical_centered(|ui| small_caps(ui, "Create Character", CAPS_SIZE, GOLD));
        ui.add_space(8.0);
        let mut changed = false;
        let mut look = false;
        ui.add_enabled_ui(!form.sending, |ui| {
            egui::Grid::new("criação-campos").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| small_caps(ui, "Name", 12.5, GOLD));
                field(ui, &mut form.name, "Character name", false, 108.0);
                ui.end_row();
                changed |= combo(ui, "raça", "Race", &mut form.race, &RACES, "Choose a race");
                let dwarf = form.race == Some(4);
                if dwarf && form.class == Some(1) {
                    form.class = Some(0);
                    changed = true;
                }
                changed |= combo(
                    ui,
                    "classe",
                    "Class",
                    &mut form.class,
                    if dwarf { &CLASSES[..1] } else { &CLASSES },
                    "Choose a class",
                );
                let mut sex = form.female.map(usize::from);
                changed |= combo(ui, "sexo", "Gender", &mut sex, &SEXES, "Choose a gender");
                form.female = sex.map(|s| s == 1);
                look |= arrows(ui, "Hairstyle", &mut form.hair_style, HAIR_STYLES);
                look |= arrows(ui, "Hair Color", &mut form.hair_color, HAIR_COLORS);
                look |= arrows(ui, "Face", &mut form.face, FACES);
            });
        });
        ui.add_space(8.0);
        Frame::new().fill(FIELD_FILL).stroke(Stroke::new(1.0, FIELD_STROKE)).inner_margin(Margin::same(8)).show(
            ui,
            |ui| {
                ui.set_width(184.0);
                ui.set_min_height(90.0);
                egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                    ui.add(Label::new(RichText::new(description).size(11.0).color(INK)).wrap());
                });
            },
        );
        ui.add_space(6.0);
        let mut zoom = false;
        ui.vertical_centered(|ui| {
            if gold_button(
                ui,
                if form.zoomed { "Zoom Out" } else { "Zoom In" },
                vec2(182.0, 20.0),
                form.female.is_some() && form.class_id().is_some(),
            ) {
                zoom = true;
            }
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if gold_button(ui, "Create", vec2(88.0, 24.0), form.complete() && !form.sending) {
                action = Some(CreateAction::Create);
            }
            ui.add_space(6.0);
            if gold_button(ui, "Cancel", vec2(88.0, 24.0), !form.sending) {
                action = Some(CreateAction::Cancel);
            }
        });
        status_line(ui, &form.status);
        if changed && action.is_none() {
            action = Some(CreateAction::Changed);
        } else if zoom && action.is_none() {
            action = Some(CreateAction::Zoom);
        } else if look && action.is_none() {
            action = Some(CreateAction::Look);
        }
    });
    action
}

pub fn loading_world(ctx: &Context, tile: &str) {
    Area::new(Id::new("carregando")).anchor(Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
        Frame::popup(ui.style()).show(ui, |ui| { ui.heading(format!("carregando {tile}…")); });
    });
}

pub fn loading(ctx: &Context, text: &str) {
    Area::new(Id::new("carregando-lobby")).anchor(Align2::CENTER_BOTTOM, vec2(0.0, -40.0)).interactable(false).show(
        ctx,
        |ui| {
            Frame::new()
                .fill(PANEL_FILL)
                .stroke(Stroke::new(1.0, PANEL_STROKE))
                .inner_margin(Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    small_caps(ui, text, CAPS_SIZE, GOLD);
                });
        },
    );
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use egui::{Event, PointerButton, Pos2, RawInput, Rect};

    pub const SCREEN: (f32, f32) = (1280.0, 720.0);

    pub fn pass<R>(ctx: &Context, events: Vec<Event>, run: impl FnOnce(&Context) -> R) -> R {
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, vec2(SCREEN.0, SCREEN.1))),
            events,
            ..Default::default()
        };
        ctx.begin_pass(input);
        let out = run(ctx);
        ctx.end_pass().textures_delta.clear();
        out
    }

    pub fn click(pos: Pos2) -> Vec<Event> {
        vec![
            Event::PointerMoved(pos),
            Event::PointerButton { pos, button: PointerButton::Primary, pressed: true, modifiers: Default::default() },
            Event::PointerButton { pos, button: PointerButton::Primary, pressed: false, modifiers: Default::default() },
        ]
    }

    pub fn click_on<R>(ctx: &Context, pos: Pos2, mut run: impl FnMut(&Context) -> R) -> R {
        pass(ctx, vec![Event::PointerMoved(pos)], &mut run);
        pass(ctx, vec![Event::PointerMoved(pos)], &mut run);
        pass(ctx, click(pos), &mut run)
    }

    pub fn settle<R>(ctx: &Context, mut run: impl FnMut(&Context) -> R) -> R {
        pass(ctx, Vec::new(), &mut run);
        pass(ctx, Vec::new(), &mut run)
    }

    pub fn rect_of(ctx: &Context, id: &'static str) -> Rect {
        ctx.memory(|m| m.area_rect(Id::new(id))).unwrap_or_else(|| panic!("área {id} não pintada"))
    }

    fn form() -> LoginForm {
        LoginForm::new("conta".into(), "senha".into(), &ServerConfig::default())
    }

    #[test]
    fn the_login_panel_is_centered_and_exit_and_log_in_are_hit_by_their_buttons() {
        let ctx = Context::default();
        let mut form = form();
        settle(&ctx, |ctx| login(ctx, &mut form));
        let rect = rect_of(&ctx, "login");
        assert!(
            (rect.center().x - SCREEN.0 / 2.0).abs() < 2.0 && (rect.center().y - SCREEN.1 / 2.0).abs() < 2.0,
            "{rect:?}"
        );
        assert!(rect.width() >= LOGIN_WIDTH - 40.0 && rect.width() <= LOGIN_WIDTH + 40.0, "{rect:?}");
        let exit = pos2(
            rect.left() + 18.0 + 96.0 + BUTTON_SIZE.x + 10.0 + BUTTON_SIZE.x / 2.0,
            rect.bottom() - 18.0 - BUTTON_SIZE.y / 2.0,
        );
        let log_in = pos2(exit.x - BUTTON_SIZE.x - 10.0, exit.y);
        let action = click_on(&ctx, log_in, |ctx| login(ctx, &mut form));
        assert_eq!(action, Some(LoginAction::Connect));
        let action = click_on(&ctx, exit, |ctx| login(ctx, &mut form));
        assert_eq!(action, Some(LoginAction::Exit));
        form.password.clear();
        assert!(!form.ready());
        let none = click_on(&ctx, log_in, |ctx| login(ctx, &mut form));
        assert_eq!(none, None);
        assert_eq!(form.status, "informe a conta e a senha");
        assert!(!pass(&ctx, Vec::new(), |ctx| {
            login(ctx, &mut form);
            ctx.memory(|m| m.area_rect(Id::new("servidor"))).is_some()
        }));
    }

    #[test]
    fn small_caps_keep_the_capital_first_and_the_glyphs_run_left_to_right_even_right_aligned() {
        let ctx = Context::default();
        let job = caps_text("Account", CAPS_SIZE, GOLD);
        assert_eq!(job.text, "ACCOUNT");
        assert_eq!(job.sections.len(), 2);
        assert_eq!(job.sections[0].byte_range.end.0 - job.sections[0].byte_range.start.0, 1);
        assert!(job.sections[0].format.font_id.size > job.sections[1].format.font_id.size);
        let xs = pass(&ctx, Vec::new(), |ctx| {
            let galley = ctx.fonts_mut(|fonts| fonts.layout_job(caps_text("Account", CAPS_SIZE, GOLD)));
            galley.rows.iter().flat_map(|row| row.glyphs.iter().map(|glyph| glyph.pos.x)).collect::<Vec<f32>>()
        });
        assert_eq!(xs.len(), 7);
        assert!(xs.windows(2).all(|pair| pair[0] < pair[1]), "{xs:?}");
        assert_eq!(xs[0], 0.0);
        let mut form = form();
        settle(&ctx, |ctx| login(ctx, &mut form));
        let rect = rect_of(&ctx, "login");
        let account = caps_text("Account", CAPS_SIZE, GOLD);
        let width = pass(&ctx, Vec::new(), |ctx| ctx.fonts_mut(|fonts| fonts.layout_job(account.clone())).size().x);
        assert!(width > 40.0 && width < rect.width() / 3.0, "{width}");
    }

    #[test]
    fn the_gear_opens_the_server_panel_which_validates_host_and_port() {
        let ctx = Context::default();
        let mut form = form();
        settle(&ctx, |ctx| login(ctx, &mut form));
        let gear = rect_of(&ctx, "engrenagem").center();
        click_on(&ctx, gear, |ctx| login(ctx, &mut form));
        assert!(form.settings_open);
        assert!(pass(&ctx, Vec::new(), |ctx| {
            login(ctx, &mut form);
            ctx.memory(|m| m.area_rect(Id::new("servidor"))).is_some()
        }));
        form.port = "abc".into();
        assert_eq!(form.config(), None);
        form.port = "2106".into();
        form.host = " ".into();
        assert_eq!(form.config(), None);
        form.host = "10.0.0.1".into();
        assert_eq!(form.config(), Some(ServerConfig { host: "10.0.0.1".into(), port: 2106 }));
    }

    #[test]
    fn the_server_list_returns_the_clicked_server_id_and_back() {
        let ctx = Context::default();
        let rows = vec![
            ServerRow { id: 1, name: "Killer 70x".into(), players: 12, max_players: 1000, online: true },
            ServerRow { id: 2, name: "Honor 25x".into(), players: 0, max_players: 500, online: false },
        ];
        settle(&ctx, |ctx| servers(ctx, &rows, ""));
        let rect = rect_of(&ctx, "servidores");
        let first = pos2(rect.center().x, rect.top() + 18.0 + 15.0 + 14.0 + 14.0);
        let second = pos2(first.x, first.y + 52.0);
        let back = pos2(rect.center().x, rect.bottom() - 18.0 - WIDE_BUTTON.y / 2.0);
        assert_eq!(click_on(&ctx, first, |ctx| servers(ctx, &rows, "")), Some(ServersAction::Choose(1)));
        assert_eq!(click_on(&ctx, second, |ctx| servers(ctx, &rows, "")), Some(ServersAction::Choose(2)));
        assert_eq!(click_on(&ctx, back, |ctx| servers(ctx, &rows, "")), Some(ServersAction::Back));
    }

    fn slots() -> Vec<CharacterSlot> {
        ["Canastra", "Sombra", "Noite"]
            .iter()
            .enumerate()
            .map(|(i, name)| CharacterSlot {
                name: name.to_string(),
                class_name: "Human Fighter".into(),
                level: 1 + i as i32,
                deleting: i == 2,
                label: Some([300.0 + 200.0 * i as f32, 280.0]),
                delete_label: Some([300.0 + 200.0 * i as f32, 268.0]),
            })
            .collect()
    }

    #[test]
    fn the_character_bar_selects_slots_enters_the_world_and_offers_create_delete_back() {
        let ctx = Context::default();
        let slots = slots();
        settle(&ctx, |ctx| characters(ctx, &slots, Some(0), "", false));
        let bar = rect_of(&ctx, "slots");
        let slot_center = |i: f32| pos2(bar.left() + 8.0 + SLOT_SIZE.x / 2.0 + i * (SLOT_SIZE.x + 6.0), bar.center().y);
        assert_eq!(
            click_on(&ctx, slot_center(1.0), |ctx| characters(ctx, &slots, Some(0), "", false)),
            Some(CharactersAction::Select(1))
        );
        assert_eq!(
            click_on(&ctx, slot_center(3.0), |ctx| characters(ctx, &slots, Some(0), "", false)),
            Some(CharactersAction::Create)
        );
        let enter = rect_of(&ctx, "entrar").center();
        assert_eq!(
            click_on(&ctx, enter, |ctx| characters(ctx, &slots, Some(0), "", false)),
            Some(CharactersAction::Enter)
        );
        assert_eq!(click_on(&ctx, enter, |ctx| characters(ctx, &slots, None, "", false)), None);
        assert_eq!(click_on(&ctx, enter, |ctx| characters(ctx, &slots, Some(0), "", true)), None);
        let back = rect_of(&ctx, "voltar").center();
        assert_eq!(
            click_on(&ctx, back, |ctx| characters(ctx, &slots, Some(0), "", false)),
            Some(CharactersAction::Back)
        );
        let delete = rect_of(&ctx, "excluir").center();
        assert_eq!(
            click_on(&ctx, delete, |ctx| characters(ctx, &slots, Some(2), "", false)),
            Some(CharactersAction::Delete)
        );
        assert!(ctx.memory(|m| m.area_rect(Id::new("cartão"))).is_some());
        let label = ctx.memory(|m| m.area_rect(Id::new(("rótulo", 1usize)))).unwrap();
        assert!((label.center().x - 500.0).abs() < 1.0 && (label.bottom() - 280.0).abs() < 1.0, "{label:?}");
    }

    #[test]
    fn the_create_form_maps_race_and_class_to_the_base_class_id_and_gates_create() {
        let mut form = CreateForm {
            name: "Nova".into(),
            race: Some(1),
            class: Some(1),
            female: Some(true),
            ..CreateForm::default()
        };
        assert_eq!(form.class_id(), Some(25));
        assert_eq!(form.class_info_id(), 4);
        assert!(form.complete());
        form.race = Some(4);
        form.class = Some(1);
        assert_eq!(form.class_id(), None);
        assert!(!form.complete());
        form.class = Some(0);
        assert_eq!(form.class_id(), Some(53));
        form.name = " ".into();
        assert!(!form.complete());
        let ctx = Context::default();
        let mut form = CreateForm::default();
        let action = settle(&ctx, |ctx| create(ctx, &mut form, "Select a race and occupation."));
        assert_eq!(action, None);
        let rect = rect_of(&ctx, "criação");
        assert!(rect.left() >= 16.0 && rect.top() >= 16.0 && rect.width() < 260.0, "{rect:?}");
        form.name = "Nova".into();
        form.race = Some(0);
        form.class = Some(0);
        form.female = Some(false);
        let create_at = pos2(rect.left() + 18.0 + 44.0, rect.bottom() - 18.0 - 12.0);
        let zoom_at = pos2(rect.center().x, create_at.y - 12.0 - 6.0 - 10.0);
        assert_eq!(click_on(&ctx, zoom_at, |ctx| create(ctx, &mut form, "")), Some(CreateAction::Zoom));
        assert_eq!(click_on(&ctx, create_at, |ctx| create(ctx, &mut form, "")), Some(CreateAction::Create));
        let cancel_at = pos2(create_at.x + 88.0 + 6.0, create_at.y);
        assert_eq!(click_on(&ctx, cancel_at, |ctx| create(ctx, &mut form, "")), Some(CreateAction::Cancel));
    }
}
