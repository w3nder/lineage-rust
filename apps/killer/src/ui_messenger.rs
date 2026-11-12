use killer_assets::client::DatTables;
use killer_ui::chat::InputKey;
use killer_ui::draw::DrawList;
use killer_ui::edit_box::FontMetrics;
use killer_ui::font::BitmapFont;
use killer_ui::messenger::{
    CHAT_CODE_MESSAGES, ChatHit, ChatWindow, FAILURE_COLOR, FriendRow, ITEM_TEXT, MessengerHit, MessengerView,
    SYSMSG_DELIVERY_FAILED, SYSMSG_SENDER_PREFIX, TaskBar, chat_position, name_filter, task_rect,
};
use killer_ui::window::{Desktop, Rect, Window, WindowId};
use killer_world::{NetCommand, World, WorldEvent};

use crate::ui_keys::UiCommand;

pub const CHAT_BASE: u32 = 0x20000;
pub const TASK_ID: WindowId = WindowId(0x1ffff);

pub struct MessengerWindows {
    pub view: MessengerView,
    pub chats: Vec<ChatWindow>,
    pub tasks: TaskBar,
    pub commands: Vec<UiCommand>,
    lookup: Vec<(u32, String)>,
    pub texts: std::collections::HashMap<u32, String>,
    pub me_name: String,
    viewport: (i32, i32),
    roll: u32,
    task_pressed: bool,
}

impl MessengerWindows {
    pub fn new(sysstrings: &std::collections::HashMap<u32, String>) -> Self {
        let lookup: Vec<(u32, String)> = sysstrings.iter().map(|(id, text)| (*id, text.clone())).collect();
        let finder = |id: u32| lookup.iter().find(|(key, _)| *key == id).map(|(_, text)| text.clone());
        Self {
            view: MessengerView::new(finder),
            chats: Vec::new(),
            tasks: TaskBar::default(),
            commands: Vec::new(),
            viewport: (1024, 768),
            roll: 0x2545_f491,
            task_pressed: false,
            texts: std::collections::HashMap::new(),
            me_name: String::new(),
            lookup,
        }
    }

    pub fn set_viewport(&mut self, viewport: (i32, i32)) {
        self.viewport = viewport;
    }

    pub fn owns(&self, id: WindowId) -> bool {
        id == WindowId::MESSENGER || id == TASK_ID || self.chat_index(id).is_some()
    }

    fn chat_index(&self, id: WindowId) -> Option<usize> {
        self.chats.iter().position(|chat| chat.id == id)
    }

    fn text(&self, id: u32) -> Option<String> {
        self.lookup.iter().find(|(key, _)| *key == id).map(|(_, text)| text.clone())
    }

    fn next_roll(&mut self) -> u32 {
        self.roll ^= self.roll << 13;
        self.roll ^= self.roll >> 17;
        self.roll ^= self.roll << 5;
        self.roll
    }

    fn show(desktop: &mut Desktop, id: WindowId, visible: bool) {
        if let Some(window) = desktop.window_mut(id) {
            window.visible = visible;
        }
        if visible && let Some(index) = desktop.windows.iter().position(|window| window.id == id) {
            let window = desktop.windows.remove(index);
            desktop.windows.push(window);
        }
    }

    pub fn sync_friends(&mut self, world: &World, rebuild: bool) {
        let rows: Vec<FriendRow> =
            world.friends.list.iter().map(|friend| FriendRow { name: friend.name.clone(), online: friend.online, object_id: friend.object_id }).collect();
        if rebuild {
            self.view.tree.rebuild(rows);
        } else {
            self.view.tree.update(rows);
        }
    }

    pub fn relabel(&mut self, lookup: impl Fn(u32) -> Option<String>) {
        let mut fresh = MessengerView::new(&lookup);
        fresh.tree.friends = std::mem::take(&mut self.view.tree.friends);
        fresh.tree.groups_expanded = self.view.tree.groups_expanded;
        fresh.tree.selected = self.view.tree.selected;
        fresh.tree.offset = self.view.tree.offset;
        self.view = fresh;
        for chat in &mut self.chats {
            chat.frame = ChatWindow::new(chat.id, &lookup).frame;
        }
    }

    pub fn set_texts(&mut self, tables: &DatTables) {
        for id in std::iter::once(SYSMSG_SENDER_PREFIX).chain(std::iter::once(SYSMSG_DELIVERY_FAILED)).chain(CHAT_CODE_MESSAGES.iter().map(|(_, message)| *message)) {
            self.texts.insert(id, crate::ui2d::system_text(Some(tables), id, &[]));
        }
    }

    pub fn world_event(&mut self, event: &WorldEvent, world: &World, desktop: &mut Desktop, font: &BitmapFont) {
        match event {
            WorldEvent::FriendList(_) => self.sync_friends(world, true),
            WorldEvent::L2Friend(_) | WorldEvent::L2FriendStatus(_) => self.sync_friends(world, false),
            WorldEvent::FriendRecvMsg(message) => self.receive(message.unknown, &message.sender, &message.message, desktop, font),
            _ => {}
        }
    }

    fn ensure_task_window(&mut self, desktop: &mut Desktop) {
        if desktop.window(TASK_ID).is_none() {
            let rect = task_rect(self.viewport);
            desktop.windows.push(Window::new(TASK_ID, rect, Rect::default()));
        }
        Self::show(desktop, TASK_ID, true);
    }

    fn system_line(&self, id: u32) -> String {
        self.texts.get(&id).cloned().unwrap_or_default()
    }

    fn sender_prefix(&self, sender: &str) -> String {
        let template = self.system_line(SYSMSG_SENDER_PREFIX);
        if template.contains("$s1") { template.replace("$s1", sender) } else { sender.to_string() }
    }

    fn acquire(&mut self, name: &str, desktop: &mut Desktop) -> usize {
        if let Some(index) = self.chats.iter().position(|chat| chat.in_use && chat.name == name) {
            return index;
        }
        let index = match self.chats.iter().position(|chat| !chat.in_use) {
            Some(index) => index,
            None => {
                let id = WindowId(CHAT_BASE + self.chats.len() as u32);
                let (rx, ry) = (self.next_roll(), self.next_roll());
                let (x, y) = chat_position(rx, ry);
                let finder = |id: u32| self.text(id);
                let chat = ChatWindow::new(id, finder);
                let rect = Rect::new(x, y, killer_ui::messenger::CHAT_SIZE.0, killer_ui::messenger::CHAT_SIZE.1);
                let mut window = Window::new(id, rect, killer_ui::messenger::CHAT_FRAME_RECT);
                window.visible = false;
                desktop.windows.push(window);
                self.chats.push(chat);
                self.chats.len() - 1
            }
        };
        self.chats[index].assign(name);
        index
    }

    pub fn open_chat(&mut self, name: &str, desktop: &mut Desktop) {
        self.ensure_task_window(desktop);
        let index = self.acquire(name, desktop);
        let id = self.chats[index].id;
        self.tasks.add(id, name, false, true);
        if let Some(task) = self.tasks.index_of(name) {
            self.tasks.tasks[task].pending = false;
        }
        self.chats[index].hidden = false;
        self.chats[index].edit.focus();
        Self::show(desktop, id, true);
    }

    pub fn receive(&mut self, code: i32, sender: &str, text: &str, desktop: &mut Desktop, font: &BitmapFont) {
        let text = name_filter(text);
        if sender.is_empty() || text.is_empty() {
            return;
        }
        self.ensure_task_window(desktop);
        let existed = self.chats.iter().any(|chat| chat.in_use && chat.name == sender);
        let index = self.acquire(sender, desktop);
        let id = self.chats[index].id;
        if !existed {
            self.chats[index].hidden = true;
            Self::show(desktop, id, false);
        }
        let visible = desktop.window(id).is_some_and(|window| window.visible);
        self.tasks.add(id, sender, true, visible);
        self.add_code_line(index, code, sender, &text, font);
    }

    fn add_code_line(&mut self, index: usize, code: i32, sender: &str, text: &str, font: &BitmapFont) {
        let metrics = FontMetrics(font);
        match code {
            0 => {
                let shown = self.sender_prefix(sender);
                self.chats[index].log.add_line(Some(sender), text, ITEM_TEXT, &|_| shown.clone(), &metrics);
            }
            code => {
                if let Some((_, message)) = CHAT_CODE_MESSAGES.iter().find(|(known, _)| *known == code) {
                    let line = self.system_line(*message);
                    self.chats[index].log.add_line(None, &line, FAILURE_COLOR, &|name| name.to_string(), &metrics);
                }
            }
        }
    }

    pub fn send(&mut self, index: usize, text: &str, font: &BitmapFont) {
        if text.is_empty() {
            return;
        }
        let name = self.chats[index].name.clone();
        let online = self.view.tree.friends.iter().any(|friend| friend.name == name && friend.online);
        let metrics = FontMetrics(font);
        if online {
            self.commands.push(UiCommand::Net(NetCommand::SendFriendMsg { message: text.to_string(), receiver: name }));
            if !self.me_name.is_empty() {
                let me_name = self.me_name.clone();
                let shown = self.sender_prefix(&me_name);
                self.chats[index].log.add_line(Some(&me_name), text, ITEM_TEXT, &|_| shown.clone(), &metrics);
            }
        } else {
            let failure = self.system_line(SYSMSG_DELIVERY_FAILED);
            let log = &mut self.chats[index].log;
            log.add_line(None, &failure, FAILURE_COLOR, &|name| name.to_string(), &metrics);
            log.add_line(None, &format!("   {text}"), FAILURE_COLOR, &|name| name.to_string(), &metrics);
        }
    }

    pub fn close_chat(&mut self, index: usize, desktop: &mut Desktop) {
        let id = self.chats[index].id;
        let name = self.chats[index].name.clone();
        Self::show(desktop, id, false);
        self.tasks.remove(&name);
        if self.tasks.tasks.is_empty() {
            Self::show(desktop, TASK_ID, false);
        }
        self.chats[index].release();
    }

    pub fn minimize_chat(&mut self, index: usize, desktop: &mut Desktop) {
        self.chats[index].hidden = true;
        self.chats[index].edit.kill_focus();
        Self::show(desktop, self.chats[index].id, false);
    }

    pub fn toggle_messenger(&mut self, desktop: &mut Desktop) {
        let visible = desktop.window(WindowId::MESSENGER).is_some_and(|window| window.visible);
        Self::show(desktop, WindowId::MESSENGER, !visible);
    }

    pub fn mouse_down(&mut self, id: WindowId, x: i32, y: i32, font: &BitmapFont) -> bool {
        if id == WindowId::MESSENGER {
            return self.view.mouse_down(x, y) == Some(MessengerHit::Page);
        }
        if id == TASK_ID {
            self.tasks.mouse_down(x, y);
            self.task_pressed = true;
            return false;
        }
        if let Some(index) = self.chat_index(id) {
            return matches!(self.chats[index].mouse_down(x, y, font), Some(ChatHit::Page));
        }
        false
    }

    pub fn mouse_move(&mut self, id: WindowId, x: i32, y: i32, font: &BitmapFont) {
        if id == WindowId::MESSENGER {
            self.view.mouse_move(y);
        } else if let Some(index) = self.chat_index(id) {
            self.chats[index].mouse_move(x, y, font);
        }
    }

    pub fn mouse_up(&mut self, over: Option<(WindowId, i32, i32)>, desktop: &mut Desktop) {
        let outside = (i32::MIN / 2, i32::MIN / 2);
        let at = |id: WindowId| over.filter(|(over_id, _, _)| *over_id == id).map_or(outside, |(_, x, y)| (x, y));
        let (x, y) = at(WindowId::MESSENGER);
        match self.view.mouse_up(x, y) {
            Some(MessengerHit::Close) => Self::show(desktop, WindowId::MESSENGER, false),
            Some(MessengerHit::Minimize) => Self::show(desktop, WindowId::MESSENGER, false),
            _ => {}
        }
        let (tx, ty) = at(TASK_ID);
        if std::mem::take(&mut self.task_pressed)
            && let Some(name) = self.tasks.mouse_up(tx, ty)
        {
            self.open_chat(&name, desktop);
        } else {
            self.tasks.pressed = None;
        }
        for index in 0..self.chats.len() {
            let id = self.chats[index].id;
            let (cx, cy) = at(id);
            match self.chats[index].mouse_up(cx, cy) {
                Some(ChatHit::Close) => self.close_chat(index, desktop),
                Some(ChatHit::Minimize) => self.minimize_chat(index, desktop),
                _ => {}
            }
        }
    }

    pub fn double_click(&mut self, id: WindowId, x: i32, y: i32, desktop: &mut Desktop) -> bool {
        if id != WindowId::MESSENGER {
            return false;
        }
        match self.view.double_click(x, y) {
            Some(name) => {
                self.open_chat(&name, desktop);
                true
            }
            None => false,
        }
    }

    pub fn wheel(&mut self, id: WindowId, x: i32, y: i32, delta: i32) -> bool {
        if id == WindowId::MESSENGER {
            return self.view.wheel(x, y, delta);
        }
        if let Some(index) = self.chat_index(id) {
            let area = killer_ui::messenger::CHAT_BOX_RECT;
            if area.contains(x, y) {
                self.chats[index].log.wheel(delta);
                return true;
            }
        }
        false
    }

    pub fn key(&mut self, key: InputKey, desktop: &Desktop, font: &BitmapFont) -> bool {
        for index in 0..self.chats.len() {
            let id = self.chats[index].id;
            if !desktop.window(id).is_some_and(|window| window.visible) || !self.chats[index].edit.focused {
                continue;
            }
            if let Some(text) = self.chats[index].key(key, font) {
                self.send(index, &text, font);
            }
            return true;
        }
        false
    }

    pub fn tick(&mut self, elapsed_ms: i32, desktop: &mut Desktop) {
        self.tasks.tick(elapsed_ms);
        for chat in &mut self.chats {
            if desktop.window(chat.id).is_some_and(|window| window.visible) {
                chat.edit.tick(elapsed_ms.max(0) as u32);
            }
        }
        if let Some(window) = desktop.window_mut(TASK_ID) {
            window.rect = task_rect(self.viewport);
        }
    }

    pub fn paint(&self, id: WindowId, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) -> bool {
        if id == WindowId::MESSENGER {
            self.view.paint(list, origin, font, alpha);
            return true;
        }
        if id == TASK_ID {
            self.tasks.paint(list, origin, font);
            return true;
        }
        match self.chat_index(id) {
            Some(index) => {
                self.chats[index].paint(list, origin, font, alpha);
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::window_owner::original_owner;

    fn font() -> BitmapFont {
        let glyphs = (32u8..=126).map(|c| (char::from(c), killer_ui::font::GlyphRect { x: i32::from(c), width: 6, y: 0, height: 12 })).collect();
        BitmapFont::new("f".into(), killer_ui::font::Glyphs { tex_width: 256, tex_height: 16, glyphs })
    }

    fn setup() -> (MessengerWindows, Desktop) {
        let mut desktop = Desktop::new((1280, 800));
        let mut window = Window::new(WindowId::MESSENGER, Rect::new(295, 158, 256, 401), Rect::new(0, 0, 256, 20));
        window.owner = original_owner(WindowId::MESSENGER);
        window.visible = false;
        desktop.windows.push(window);
        let strings = std::collections::HashMap::new();
        let mut windows = MessengerWindows::new(&strings);
        for id in [SYSMSG_SENDER_PREFIX, SYSMSG_DELIVERY_FAILED, 1354, 1355, 1356, 1357] {
            windows.texts.insert(id, format!("t{id} $s1"));
        }
        (windows, desktop)
    }

    fn visible(desktop: &Desktop, id: WindowId) -> bool {
        desktop.window(id).is_some_and(|window| window.visible)
    }

    #[test]
    fn a_message_for_an_unknown_name_creates_a_hidden_window_and_a_pending_task() {
        let (mut windows, mut desktop) = setup();
        windows.receive(0, "Ana", "hi", &mut desktop, &font());
        assert_eq!(windows.chats.len(), 1);
        assert!(!visible(&desktop, windows.chats[0].id));
        assert!(visible(&desktop, TASK_ID));
        assert!(windows.tasks.tasks[0].pending);
        assert!(windows.chats[0].log.lines().len() >= 2);
        windows.receive(0, "Ana", "again", &mut desktop, &font());
        assert_eq!(windows.chats.len(), 1);
    }

    #[test]
    fn opening_the_conversation_shows_it_clears_the_pending_flag_and_reuses_the_window() {
        let (mut windows, mut desktop) = setup();
        windows.receive(0, "Ana", "hi", &mut desktop, &font());
        windows.open_chat("Ana", &mut desktop);
        assert!(visible(&desktop, windows.chats[0].id));
        assert!(!windows.tasks.tasks[0].pending);
        windows.receive(0, "Ana", "more", &mut desktop, &font());
        assert!(!windows.tasks.tasks[0].pending);
        windows.minimize_chat(0, &mut desktop);
        windows.receive(0, "Ana", "third", &mut desktop, &font());
        assert!(windows.tasks.tasks[0].pending);
    }

    #[test]
    fn closing_returns_the_window_to_the_pool_and_hides_the_bar_when_the_last_task_goes() {
        let (mut windows, mut desktop) = setup();
        windows.open_chat("Ana", &mut desktop);
        windows.close_chat(0, &mut desktop);
        assert!(!windows.chats[0].in_use);
        assert!(windows.chats[0].log.lines().is_empty());
        assert!(!visible(&desktop, TASK_ID));
        windows.open_chat("Bo", &mut desktop);
        assert_eq!(windows.chats.len(), 1);
        assert_eq!(windows.chats[0].name, "Bo");
    }

    #[test]
    fn the_failure_codes_print_one_red_line_and_unknown_codes_are_ignored() {
        let (mut windows, mut desktop) = setup();
        windows.receive(145, "Ana", "x", &mut desktop, &font());
        assert_eq!(windows.chats[0].log.lines().len(), 1);
        assert_eq!(windows.chats[0].log.lines()[0].1, FAILURE_COLOR);
        windows.receive(9999, "Ana", "x", &mut desktop, &font());
        assert_eq!(windows.chats[0].log.lines().len(), 1);
    }

    #[test]
    fn sending_to_an_offline_friend_sends_nothing_and_prints_the_two_red_lines() {
        let (mut windows, mut desktop) = setup();
        windows.view.tree.rebuild(vec![FriendRow { name: "Ana".into(), online: false, object_id: 1 }]);
        windows.open_chat("Ana", &mut desktop);
        windows.send(0, "hello", &font());
        assert!(windows.commands.is_empty());
        let lines = windows.chats[0].log.lines();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1].0, "   hello");
    }

    #[test]
    fn sending_to_an_online_friend_emits_the_request() {
        let (mut windows, mut desktop) = setup();
        windows.view.tree.rebuild(vec![FriendRow { name: "Ana".into(), online: true, object_id: 1 }]);
        windows.open_chat("Ana", &mut desktop);
        windows.send(0, "hello", &font());
        assert!(matches!(
            windows.commands.as_slice(),
            [UiCommand::Net(NetCommand::SendFriendMsg { message, receiver })] if message == "hello" && receiver == "Ana"
        ));
    }

    #[test]
    fn alt_y_toggles_the_messenger() {
        let (mut windows, mut desktop) = setup();
        windows.toggle_messenger(&mut desktop);
        assert!(visible(&desktop, WindowId::MESSENGER));
        windows.toggle_messenger(&mut desktop);
        assert!(!visible(&desktop, WindowId::MESSENGER));
    }
}
