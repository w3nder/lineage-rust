use killer_ui::chat::InputKey;
use killer_ui::macros::{MacroCommandView, MacroEditView, MacroHit, MacroInfoView, MacroListView, MacroView};
use killer_ui::window::{Desktop, WindowId};
use killer_world::{NetCommand, World, WorldEvent};
use std::collections::HashMap;

use crate::ui_keys::UiCommand;

pub struct MacroTables<'a> {
    pub skillname: &'a HashMap<(u32, u32), l2_dat::skillname::SkillName>,
    pub commandname: &'a HashMap<String, u32>,
}

pub const MACRO_LINES_INVALID_SYSMSG: u32 = 0x32a;
pub const MACRO_NAME_EMPTY_SYSMSG: u32 = 0x346;
pub const MACRO_NAME_USED_SYSMSG: u32 = 0x347;
pub const MACRO_MESSAGE_COLOR: u32 = 0xffb09b79;

/// "You cannot register more than 24 macros." — mensagem que o botão de adicionar manda quando
/// a lista está cheia (`pushl $0x31d` em `0x100a90c7`).
pub const MACRO_LIMIT_SYSMSG: u32 = 0x31d;

pub struct MacroWindow {
    pub list: MacroListView,
    pub info: MacroInfoView,
    pub edit: MacroEditView,
    pub commands: Vec<UiCommand>,
    /// Macro que o clique na lista mandou executar, colhido pelo `sync` do `KeyWindows`.
    pub run: Option<i32>,
    /// Macro arrastado pra lixeira, esperando a confirmação `0x33c`.
    pub confirm_delete: Option<(i32, String)>,
    pub shortcut_macros: Vec<bool>,
    pub builder: bool,
    /// Ícone que acompanha o cursor enquanto se arrasta um macro. O original troca a própria
    /// imagem do cursor (`FUN_1002b5b0(this, 4, ícone)` → `FUN_100399d0`) já no primeiro
    /// `OnMouseMove` do arrasto, antes de acender os destinos.
    pub drag_icon: Option<String>,
    last_tick: Option<killer_platform::time::Instant>,
}

impl MacroWindow {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self { list: MacroListView::new(&lookup), info: MacroInfoView::new(&lookup), edit: MacroEditView::new(lookup), commands: Vec::new(), run: None, confirm_delete: None, shortcut_macros: Vec::new(), builder: false, drag_icon: None, last_tick: None }
    }

    pub fn set_sysstrings(&mut self, lookup: impl Fn(u32) -> Option<String>) {
        let font = self.edit.font.take();
        let info_font = self.info.font.take();
        *self = Self::new(lookup);
        self.edit.font = font;
        self.info.font = info_font;
    }

    pub fn tick(&mut self, now: killer_platform::time::Instant) {
        if let Some(last) = self.last_tick.replace(now) {
            let elapsed = now.saturating_duration_since(last).as_millis().min(u128::from(u32::MAX)) as u32;
            self.edit.tick(elapsed);
            self.info.edit.tick(elapsed);
        }
    }

    pub fn open(&mut self, desktop: &mut Desktop) {
        show(desktop, WindowId::MACRO_LIST, true);
    }

    pub fn world_event(
        &mut self,
        event: &WorldEvent,
        world: &World,
        label: impl Fn(&l2_net::game::packets::MacroCommand) -> String,
        desktop: &mut Desktop,
    ) -> bool {
        if !matches!(event, WorldEvent::SendMacroList(_)) { return false; }
        let macros = world.macros.macros.iter().map(|macro_entry| MacroView {
            id: macro_entry.id,
            name: macro_entry.name.clone(),
            description: macro_entry.description.clone(),
            acronym: macro_entry.acronym.clone(),
            icon: macro_entry.icon,
            commands: macro_entry.commands.iter().map(|command| MacroCommandView {
                kind: command.kind,
                skill_id: command.skill_id,
                shortcut_id: command.shortcut_id,
                command: command.command.clone(),
                label: label(command),
            }).collect(),
        }).collect();
        self.list.set_macros(macros, usize::from(world.macros.total_count));
        let _ = desktop;
        true
    }

    pub fn mouse_down(
        &mut self,
        id: WindowId,
        x: i32,
        y: i32,
        _desktop: &mut Desktop,
    ) {
        match id {
            WindowId::MACRO_LIST => {
                self.list.pressed = self.list.hit(x, y);
                // `NCMacroArrayWnd::OnLButtonDown @ 0x100a9760`: apertar numa célula só guarda o
                // índice escolhido, marca o macro (`+0xa38 = 1`, `FUN_100a78a0`) e repinta. O que
                // acontece de fato é decidido ao soltar, pelo lugar onde o botão foi solto.
                if let Some(MacroHit::Macro(index)) = self.list.pressed {
                    self.list.selected = Some(index);
                    self.list.dragging = Some(index);
                    self.info.macro_entry = self.list.macros.get(index).cloned();
                    self.drag_icon = self.list.macros.get(index).map(killer_ui::macros::macro_icon);
                }
            }
            WindowId::MACRO_INFO => {
                self.info.pressed = self.info.hit(x, y);
                if self.info.pressed.is_none() {
                    self.info.mouse_down_edit(x, y);
                } else {
                    self.info.edit.kill_focus();
                }
            }
            WindowId::MACRO_EDIT => {
                self.info.edit.kill_focus();
                self.edit.pressed = None;
                if self.edit.mouse_down_box(x, y) { return; }
                if self.edit.scroll_down(x, y) { return; }
                self.edit.pressed = self.edit.hit(x, y).filter(|hit| {
                    matches!(hit, MacroHit::MoreInfo | MacroHit::Prev | MacroHit::Next | MacroHit::InfoOk | MacroHit::InfoCancel | MacroHit::Close | MacroHit::Help)
                });
            }
            _ => {}
        }
    }

    pub fn mouse_up(&mut self, id: WindowId, x: i32, y: i32, tables: &MacroTables<'_>, desktop: &mut Desktop) {
        if id == WindowId::MACRO_LIST {
            let pressed = self.list.pressed.take();
            self.drag_icon = None;
            let dragged = self.list.dragging.take().and_then(|index| self.list.macros.get(index).cloned());
            let released = self.list.hit(x, y);
            match (pressed, released) {
                (Some(MacroHit::Close), Some(MacroHit::Close)) => show(desktop, id, false),
                // `NCMacroArrayWnd::OnLButtonUp @ 0x100a9a40`: soltando sobre a MESMA célula em
                // que apertou, o original põe `+0xa34 = 1` no macro (`0x100a9aec`) — a mesma
                // flag que o `NConsoleWnd::RunMacro @ 0x10074eb0` liga. Ou seja, clicar executa.
                (Some(MacroHit::Macro(from)), Some(MacroHit::Macro(to))) if from == to => {
                    if let Some(macro_entry) = dragged {
                        self.run = Some(macro_entry.id);
                    }
                }
                // Soltar sobre o ícone de edição (`FUN_100a79a0`, `0x100a9bc7`) copia nome,
                // sigla e comandos do macro arrastado e abre o editor (`0x100a9e96`).
                (Some(MacroHit::Macro(_)), Some(MacroHit::EditIcon)) => self.open_editor(dragged, desktop),
                // Soltar sobre a lixeira (`FUN_100a7940`, `0x100a9b01`) pede confirmação com a
                // mensagem `0x33c` antes de apagar.
                (Some(MacroHit::Macro(_)), Some(MacroHit::Trash)) => {
                    if let Some(macro_entry) = dragged {
                        self.confirm_delete = Some((macro_entry.id, macro_entry.name.clone()));
                    }
                }
                // O botão de adicionar abre o editor vazio (`0x100a9060`), recusando a partir do
                // 24º macro (`cmpl $0x18` em `0x100a90b1`, mensagem `0x31d`).
                (Some(MacroHit::Add), Some(MacroHit::Add)) => {
                    if self.list.macros.len() >= killer_ui::macros::MACROS_MAX {
                        self.commands.push(UiCommand::SystemMessage(MACRO_LIMIT_SYSMSG));
                    } else {
                        self.list.selected = None;
                        self.open_editor(None, desktop);
                    }
                }
                (Some(MacroHit::Help), Some(MacroHit::Help)) => {
                    self.commands.push(UiCommand::ToggleHelp { macro_help: true })
                }
                _ => {}
            }
        }
        if id == WindowId::MACRO_INFO {
            let pressed = self.info.pressed.take();
            if pressed.is_some() && self.info.hit(x, y) == pressed {
                match pressed {
                    Some(MacroHit::InfoOk) => self.info_ok(desktop),
                    Some(MacroHit::InfoCancel) => show(desktop, WindowId::MACRO_INFO, false),
                    _ => {}
                }
            }
        }
        if id == WindowId::MACRO_EDIT {
            let pressed = self.edit.pressed.take();
            if pressed.is_some() && self.edit.hit(x, y) == pressed {
                self.edit_button(pressed, tables, desktop);
            }
        }
    }

    fn info_ok(&mut self, desktop: &mut Desktop) {
        if !self.info.accept() {
            self.message(killer_ui::macros::INFO_TOO_LONG_SYSMSG);
            return;
        }
        show(desktop, WindowId::MACRO_INFO, false);
    }

    fn open_editor(&mut self, macro_entry: Option<MacroView>, desktop: &mut Desktop) {
        self.edit.set_macro(macro_entry);
        let description = self.edit.description.text.clone();
        if description.chars().count() <= killer_ui::macros::INFO_DESCRIPTION_MAX_CHARS {
            self.info.stored = description.clone();
            self.info.set_description(&description);
        }
        self.edit.focus(killer_ui::macros::EditFocus::Command(0));
        show(desktop, WindowId::MACRO_INFO, false);
        show(desktop, WindowId::MACRO_EDIT, true);
    }

    pub fn clear(&mut self, desktop: &mut Desktop) {
        self.list.macros.clear();
        self.list.selected = None;
        self.info.macro_entry = None;
        self.edit.set_macro(None);
        show(desktop, WindowId::MACRO_LIST, false);
        show(desktop, WindowId::MACRO_INFO, false);
        show(desktop, WindowId::MACRO_EDIT, false);
    }

    fn edit_button(&mut self, hit: Option<MacroHit>, tables: &MacroTables<'_>, desktop: &mut Desktop) {
        match hit {
            Some(MacroHit::Prev) => self.edit.select_previous_icon(),
            Some(MacroHit::Next) => self.edit.select_next_icon(),
            Some(MacroHit::MoreInfo) => {
                if desktop.window(WindowId::MACRO_INFO).is_some_and(|window| window.visible) {
                    show(desktop, WindowId::MACRO_INFO, false);
                } else {
                    self.info.macro_entry = Some(self.edit.result());
                    self.info.open();
                    show(desktop, WindowId::MACRO_INFO, true);
                }
            }
            Some(MacroHit::Help) => self.commands.push(UiCommand::ToggleHelp { macro_help: true }),
            Some(MacroHit::InfoCancel) => hide_editor(desktop),
            Some(MacroHit::Close) => {
                hide_editor(desktop);
            }
            Some(MacroHit::InfoOk) => self.save(tables, desktop),
            _ => {}
        }
    }

    fn save(&mut self, tables: &MacroTables<'_>, desktop: &mut Desktop) {
        let lines: Vec<String> = (0..killer_ui::macros::MACRO_COMMANDS_MAX).map(|line| self.edit.line_text(line)).collect();
        let shortcut_macros = &self.shortcut_macros;
        let shortcut_is_macro = |page: i32, slot: i32| {
            shortcut_macros.get(((page - 1) * 12 + (slot - 1)) as usize).copied().unwrap_or(false)
        };
        if !crate::ui_keys::macro_validate_lines(&lines, tables.commandname, &shortcut_is_macro, self.builder) {
            self.message(MACRO_LINES_INVALID_SYSMSG);
            return;
        }
        let mut view = self.edit.result();
        view.description = self.info.stored.clone();
        if view.name.is_empty() {
            self.message(MACRO_NAME_EMPTY_SYSMSG);
            return;
        }
        let selected = self.list.selected.filter(|index| *index < self.list.macros.len());
        let used = self.list.macros.iter().any(|other| {
            let same = selected.is_some_and(|index| self.list.macros[index].id == other.id);
            !same && !other.name.is_empty() && other.name.to_lowercase() == view.name.to_lowercase()
        });
        if used {
            self.message(MACRO_NAME_USED_SYSMSG);
            return;
        }
        hide_editor(desktop);
        let id = selected.map_or(0, |index| self.list.macros[index].id);
        let (count, mut commands) = crate::ui_keys::macro_save_records(&lines, tables.skillname, tables.commandname, self.builder);
        commands.truncate(count);
        self.commands.push(UiCommand::Net(NetCommand::MakeMacro(Box::new(l2_net::game::packets::Macro {
            id,
            name: view.name,
            description: view.description,
            acronym: view.acronym,
            icon: view.icon,
            commands,
        }))));
    }

    fn message(&mut self, id: u32) {
        self.commands.push(UiCommand::ColoredSystemMessage { id, color: MACRO_MESSAGE_COLOR });
    }

    pub fn key(&mut self, key: InputKey, desktop: &mut Desktop) -> bool {
        if desktop.window(WindowId::MACRO_INFO).is_some_and(|window| window.visible) && self.info.key(key) {
            return true;
        }
        if !desktop.window(WindowId::MACRO_EDIT).is_some_and(|window| window.visible) { return false; }
        if key == InputKey::Escape {
            hide_editor(desktop);
        } else {
            self.edit.key(key);
        }
        true
    }

    pub fn take_commands(&mut self) -> Vec<UiCommand> { std::mem::take(&mut self.commands) }
}

fn hide_editor(desktop: &mut Desktop) {
    show(desktop, WindowId::MACRO_INFO, false);
    show(desktop, WindowId::MACRO_EDIT, false);
}

fn show(desktop: &mut Desktop, id: WindowId, value: bool) {
    if let Some(window) = desktop.window_mut(id) { window.visible = value; }
    if value && let Some(index) = desktop.windows.iter().position(|window| window.id == id) {
        let window = desktop.windows.remove(index);
        desktop.windows.push(window);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::macros::{EditFocus, EDIT_SAVE_RECT};
    use killer_ui::window::{Rect, Window};

    fn names() -> HashMap<String, u32> {
        [("attack".to_string(), 8u32)].into_iter().collect()
    }

    fn setup() -> (MacroWindow, Desktop) {
        let mut desktop = Desktop::new((800, 600));
        for id in [WindowId::MACRO_EDIT, WindowId::MACRO_INFO] {
            desktop.windows.push(Window::new(id, Rect::new(0, 0, 256, 401), Rect::new(0, 0, 256, 20)));
        }
        let mut window = MacroWindow::new(|_| None);
        window.edit.set_font(&crate::ui2d::tests::small_font());
        (window, desktop)
    }

    fn type_into(window: &mut MacroWindow, focus: EditFocus, text: &str) {
        window.edit.focus(focus);
        for c in text.chars() {
            window.edit.key(InputKey::Char(c));
        }
    }

    fn press_save(window: &mut MacroWindow, desktop: &mut Desktop) -> Vec<UiCommand> {
        let skills = HashMap::new();
        let commands = names();
        let tables = MacroTables { skillname: &skills, commandname: &commands };
        window.mouse_down(WindowId::MACRO_EDIT, EDIT_SAVE_RECT.x + 2, EDIT_SAVE_RECT.y + 2, desktop);
        window.mouse_up(WindowId::MACRO_EDIT, EDIT_SAVE_RECT.x + 2, EDIT_SAVE_RECT.y + 2, &tables, desktop);
        window.take_commands()
    }

    fn macro_entry(id: i32, name: &str) -> killer_ui::macros::MacroView {
        killer_ui::macros::MacroView { id, name: name.into(), description: String::new(), acronym: String::new(), icon: 1, commands: Vec::new() }
    }

    #[test]
    fn on_save_100ab8d0_checks_lines_then_name_then_other_macros_before_sending() {
        let (mut window, mut desktop) = setup();
        window.list.macros = vec![macro_entry(10, "Buff"), macro_entry(11, "Heal")];
        window.edit.set_macro(None);
        show(&mut desktop, WindowId::MACRO_EDIT, true);
        assert_eq!(press_save(&mut window, &mut desktop), [UiCommand::ColoredSystemMessage { id: 0x32a, color: 0xffb09b79 }]);
        type_into(&mut window, EditFocus::Command(0), "/attack");
        assert_eq!(press_save(&mut window, &mut desktop), [UiCommand::ColoredSystemMessage { id: 0x346, color: 0xffb09b79 }]);
        type_into(&mut window, EditFocus::Name, "HEAL");
        assert_eq!(press_save(&mut window, &mut desktop), [UiCommand::ColoredSystemMessage { id: 0x347, color: 0xffb09b79 }]);
        assert!(desktop.window(WindowId::MACRO_EDIT).unwrap().visible);
        window.list.selected = Some(1);
        let sent = press_save(&mut window, &mut desktop);
        let [UiCommand::Net(NetCommand::MakeMacro(saved))] = sent.as_slice() else { panic!("{sent:?}") };
        assert_eq!((saved.id, saved.name.as_str(), saved.commands.len()), (11, "HEAL", 1));
        assert_eq!((saved.commands[0].index, saved.commands[0].kind, saved.commands[0].command.as_str()), (1, 3, "/attack"));
        assert!(!desktop.window(WindowId::MACRO_EDIT).unwrap().visible);
    }

    #[test]
    fn builder_codes_100aa260_only_save_for_a_player_with_builder_level() {
        for (builder, sent) in [(false, false), (true, true)] {
            let (mut window, mut desktop) = setup();
            window.builder = builder;
            window.edit.set_macro(None);
            show(&mut desktop, WindowId::MACRO_EDIT, true);
            type_into(&mut window, EditFocus::Command(0), "//gmmenu");
            type_into(&mut window, EditFocus::Name, "Gm");
            let commands = press_save(&mut window, &mut desktop);
            if sent {
                let [UiCommand::Net(NetCommand::MakeMacro(saved))] = commands.as_slice() else { panic!("{commands:?}") };
                assert_eq!(saved.commands[0].command, "//gmmenu");
            } else {
                assert_eq!(commands, [UiCommand::ColoredSystemMessage { id: 0x32a, color: 0xffb09b79 }]);
            }
        }
    }

    #[test]
    fn edit_buttons_act_on_release_inside_and_help_and_info_follow_the_dll() {
        let (mut window, mut desktop) = setup();
        window.edit.set_macro(None);
        show(&mut desktop, WindowId::MACRO_EDIT, true);
        let help = killer_ui::macros::EDIT_HELP_RECT;
        window.mouse_down(WindowId::MACRO_EDIT, help.x + 2, help.y + 2, &mut desktop);
        assert!(window.take_commands().is_empty());
        let skills = HashMap::new();
        let commands = names();
        let tables = MacroTables { skillname: &skills, commandname: &commands };
        window.mouse_up(WindowId::MACRO_EDIT, help.x + 2, help.y + 2, &tables, &mut desktop);
        assert_eq!(window.take_commands(), [UiCommand::ToggleHelp { macro_help: true }]);
        show(&mut desktop, WindowId::MACRO_INFO, false);
        let info = killer_ui::macros::EDIT_MORE_INFO_RECT;
        for visible in [true, false] {
            window.mouse_down(WindowId::MACRO_EDIT, info.x + 2, info.y + 2, &mut desktop);
            window.mouse_up(WindowId::MACRO_EDIT, info.x + 2, info.y + 2, &tables, &mut desktop);
            assert_eq!(desktop.window(WindowId::MACRO_INFO).unwrap().visible, visible);
        }
        window.mouse_down(WindowId::MACRO_EDIT, help.x + 2, help.y + 2, &mut desktop);
        window.mouse_up(WindowId::MACRO_EDIT, 100, 100, &tables, &mut desktop);
        assert!(window.take_commands().is_empty());
    }
}
