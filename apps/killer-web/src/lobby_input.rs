#[derive(Default)]
pub struct Input {
    events: Vec<egui::Event>,
    modifiers: egui::Modifiers,
}

impl Input {
    pub fn pointer(&mut self, kind: &str, x: f32, y: f32, button: i32) {
        let pos = egui::pos2(x, y);
        match kind {
            "move" => self.events.push(egui::Event::PointerMoved(pos)),
            "down" | "up" => {
                self.events.push(egui::Event::PointerMoved(pos));
                let button = match button {
                    0 => egui::PointerButton::Primary,
                    1 => egui::PointerButton::Middle,
                    2 => egui::PointerButton::Secondary,
                    _ => return,
                };
                self.events.push(egui::Event::PointerButton {
                    pos,
                    button,
                    pressed: kind == "down",
                    modifiers: self.modifiers,
                });
            }
            "leave" | "cancel" => self.events.push(egui::Event::PointerGone),
            _ => {}
        }
    }

    pub fn keyboard(&mut self, text: &str, vk: u32, alt: bool, pressed: bool, repeat: bool) {
        self.modifiers.alt = alt;
        if vk == 0x11 {
            self.modifiers.ctrl = pressed;
        }
        if vk == 0x10 {
            self.modifiers.shift = pressed;
        }
        if matches!(vk, 0x5b | 0x5c) {
            self.modifiers.mac_cmd = pressed;
        }
        self.modifiers.command = self.modifiers.ctrl || self.modifiers.mac_cmd;
        self.events.push(egui::Event::ModifiersChanged(self.modifiers));
        if let Some(key) = egui::Key::from_name(text) {
            self.events.push(egui::Event::Key { key, physical_key: None, pressed, repeat, modifiers: self.modifiers });
        }
        if pressed && !self.modifiers.command && !alt && text.chars().count() == 1 {
            self.events.push(egui::Event::Text(text.into()));
        }
    }

    pub fn paste(&mut self, text: String) {
        self.events.push(egui::Event::Paste(text));
    }

    pub fn wheel(&mut self, ticks: f32) {
        self.events.push(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: egui::vec2(0.0, ticks),
            phase: egui::TouchPhase::Move,
            modifiers: self.modifiers,
        });
    }

    pub fn take(&mut self, size: (u32, u32), scale: f32, time: f64) -> egui::RawInput {
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(size.0 as f32 / scale, size.1 as f32 / scale),
            )),
            time: Some(time),
            events: std::mem::take(&mut self.events),
            ..Default::default()
        }
    }

    pub fn focus(&mut self, focused: bool) {
        if !focused {
            self.modifiers = Default::default();
            self.events.push(egui::Event::ModifiersChanged(self.modifiers));
        }
        self.events.push(egui::Event::WindowFocused(focused));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_input_preserves_logical_positions_buttons_typing_paste_and_focus() {
        let mut input = Input::default();
        input.pointer("down", 320.0, 180.0, 0);
        input.pointer("up", 320.0, 180.0, 0);
        input.keyboard("A", 65, false, true, false);
        input.keyboard("Tab", 9, false, true, false);
        input.paste("Teste".into());
        let frame = input.take((1280, 720), 2.0, 1.0);
        assert_eq!(frame.screen_rect.unwrap().size(), egui::vec2(640.0, 360.0));
        assert!(frame.events.iter().any(
            |event| matches!(event,egui::Event::PointerButton {pos,pressed:true,..} if *pos==egui::pos2(320.0,180.0))
        ));
        assert!(frame.events.iter().any(|event| matches!(event,egui::Event::Text(text) if text=="A")));
        assert!(
            frame
                .events
                .iter()
                .any(|event| matches!(event, egui::Event::Key { key: egui::Key::Tab, pressed: true, .. }))
        );
        assert!(frame.events.iter().any(|event| matches!(event,egui::Event::Paste(text) if text=="Teste")));
        assert!(input.take((1280, 720), 2.0, 2.0).events.is_empty());
        input.keyboard("Control", 0x11, false, true, false);
        input.keyboard("a", 65, false, true, false);
        assert!(!input.events.iter().any(|event| matches!(event, egui::Event::Text(_))));
        input.focus(false);
        assert!(!input.modifiers.command);
        assert!(matches!(input.events.last(), Some(egui::Event::WindowFocused(false))));
    }

    #[test]
    fn browser_typing_and_enter_reach_egui_text_fields() {
        let context = egui::Context::default();
        let mut input = Input::default();
        let mut text = String::new();
        let mut submitted = false;
        for frame in 0..4 {
            if frame == 1 {
                input.keyboard("W", 87, false, true, false);
                input.paste("ender".into());
            }
            if frame == 2 {
                input.keyboard("Enter", 13, false, true, false);
            }
            context.begin_pass(input.take((1280, 720), 1.0, frame as f64));
            egui::Window::new("entrada").show(&context, |ui| {
                let response = ui.text_edit_singleline(&mut text);
                if frame == 0 {
                    response.request_focus();
                }
                submitted |= response.lost_focus() && ui.input(|state| state.key_pressed(egui::Key::Enter));
            });
            context.end_pass().textures_delta.clear();
        }
        assert_eq!(text, "Wender");
        assert!(submitted);
    }
}
