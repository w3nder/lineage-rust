use crate::color::{Argb, BUTTON_TEXT};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{BUTTON_SIZE, BUTTON_Y, CLOSE_INSET, FrameCtrl, TITLE_HEIGHT};
use crate::html::{self, Document, Line, WidgetKind};
use crate::window::Rect;

pub const NPC_VIEWER_SIZE: (i32, i32) = (310, 401);

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    crate::mainwnd::default_position(viewport)
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    crate::mainwnd::initial_position(viewport)
}

pub const NPC_HTML_BACK: &str = "L2UI.etcwnd.NPCHtml_BACK";
pub const NPC_HTML_BACK_SIZE: (i32, i32) = (512, 256);
pub const CONTENT_POS: (i32, i32) = (7, 30);
pub const CONTENT_INSET: (i32, i32) = (14, 37);
pub const CHAT_TITLE_SYSSTRING: u32 = 444;
pub const DEFAULT_TITLE: &str = "Chat";
pub const HTML_BUTTON_TEXT: Argb = BUTTON_TEXT;
pub const HTML_WHEEL_STEP: i32 = 39;
pub const HTML_WRAP_MARGIN: i32 = 16;

pub fn content_size() -> (i32, i32) {
    (NPC_VIEWER_SIZE.0 - CONTENT_INSET.0, NPC_VIEWER_SIZE.1 - CONTENT_INSET.1)
}

pub fn content_rect() -> Rect {
    let (w, h) = content_size();
    Rect::new(CONTENT_POS.0, CONTENT_POS.1, w, h)
}

pub fn close_button() -> Rect {
    Rect::new(NPC_VIEWER_SIZE.0 - CLOSE_INSET, BUTTON_Y, BUTTON_SIZE, BUTTON_SIZE)
}

pub fn grip() -> Rect {
    FrameCtrl::titled(DEFAULT_TITLE, true).grip(NPC_VIEWER_SIZE.0, NPC_VIEWER_SIZE.1)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcViewer {
    pub title: String,
    pub doc: Document,
    pub lines: Vec<Line>,
    pub height: i32,
    pub scroll: i32,
    pub form: crate::html_form::HtmlForm,
    pub bar: crate::scroll::ScrollWnd,
    pub bar_pressed: Option<crate::scroll::ScrollHit>,
    pub pressed: Option<(i32, i32)>,
    pub pointer: Option<(i32, i32)>,
    pub anim_on: std::cell::Cell<bool>,
    pub anim_start_ms: std::cell::Cell<Option<u32>>,
    pub clock_ms: std::cell::Cell<u32>,
}

pub const TEXT_ANIM_STEP: f32 = 0.007;

pub const LINE_STEP: i32 = HTML_WHEEL_STEP / 3;

pub fn round_to_line(value: i32) -> i32 {
    (LINE_STEP - 1 + value) / LINE_STEP * LINE_STEP
}

impl NpcViewer {
    pub fn new(title: &str, html_text: &str, font: &BitmapFont) -> NpcViewer {
        NpcViewer::with_markers(title, html_text, font, &|_, _| None)
    }

    pub fn with_markers(title: &str, html_text: &str, font: &BitmapFont, resolve: &dyn Fn(html::Marker, i32) -> Option<String>) -> NpcViewer {
        let doc = html::parse_with(html_text, resolve);
        let lines = html::layout(&doc, content_size().0 - HTML_WRAP_MARGIN, font);
        let height = html::height(&lines, font);
        let text_anim = doc.text_anim();
        let mut form = crate::html_form::HtmlForm::from_document(&doc);
        form.attach(&lines);
        let (w, h) = content_size();
        let mut bar = crate::scroll::ScrollWnd::new(w, h);
        bar.slider.set_range(height, h);
        NpcViewer { title: title.to_string(), doc, lines, height, scroll: 0, form, bar, bar_pressed: None, pressed: None, pointer: None, anim_on: std::cell::Cell::new(text_anim), anim_start_ms: std::cell::Cell::new(None), clock_ms: std::cell::Cell::new(0) }
    }

    pub fn max_scroll(&self) -> i32 {
        (self.height - content_size().1).max(0)
    }

    pub fn scroll_by(&mut self, dy: i32) {
        self.scroll = (self.scroll + dy).clamp(0, self.max_scroll());
        self.bar.slider.set_pos(self.scroll);
    }

    pub fn scroll_press(&mut self, x: i32, y: i32) -> bool {
        let content = content_rect();
        if !content.contains(x, y) || self.height <= content.h {
            return false;
        }
        let Some(hit) = self.bar.hit(content.w, content.h, x - content.x, y - content.y) else { return false };
        self.bar_pressed = Some(hit);
        match hit {
            crate::scroll::ScrollHit::Up => {
                self.bar.up_pressed = true;
                self.scroll_by(-LINE_STEP);
            }
            crate::scroll::ScrollHit::Down => {
                self.bar.down_pressed = true;
                self.scroll_by(LINE_STEP);
            }
            crate::scroll::ScrollHit::Slider(at) => {
                self.bar.slider.mouse_down(at);
            }
        }
        true
    }

    pub fn scroll_move(&mut self, y: i32) -> bool {
        if !self.bar.slider.dragging {
            return false;
        }
        let content = content_rect();
        let track = crate::scroll::scroll_rects(content.w, content.h).slider;
        if let Some(pos) = self.bar.slider.mouse_move(y - content.y - track.y) {
            self.scroll = round_to_line(pos);
        }
        true
    }

    pub fn stop_text_anim(&self) {
        self.anim_on.set(false);
        self.anim_start_ms.set(None);
    }

    pub fn set_clock_ms(&self, ms: u32) {
        self.clock_ms.set(ms);
    }

    pub fn scroll_release(&mut self) {
        self.bar.up_pressed = false;
        self.bar.down_pressed = false;
        self.bar.slider.mouse_up();
        self.bar_pressed = None;
    }

    pub fn combo_press(&mut self, x: i32, y: i32) -> bool {
        let content = content_rect();
        self.form.combo_press(&self.lines, self.scroll, x - content.x, y - content.y)
    }

    pub fn combo_release(&mut self, x: i32, y: i32) -> bool {
        let content = content_rect();
        self.form.combo_release(&self.lines, self.scroll, x - content.x, y - content.y)
    }

    pub fn combo_move(&mut self, x: i32, y: i32) {
        let content = content_rect();
        self.form.combo_move(&self.lines, self.scroll, x - content.x, y - content.y);
    }

    pub fn edit_top_at(&self, x: i32, y: i32) -> Option<i32> {
        let content = content_rect();
        if !content.contains(x, y) {
            return None;
        }
        let (px, py) = (x - content.x, y - content.y + self.scroll);
        self.lines.iter().find_map(|line| {
            line.widgets
                .iter()
                .find(|widget| matches!(widget.kind, WidgetKind::Edit { .. }) && px >= widget.x && px < widget.x + widget.width && py >= line.y && py < line.y + widget.height)
                .map(|_| line.y - self.scroll)
        })
    }

    pub fn scroll_to_edit(&mut self, local_top: i32) {
        if local_top < 0 {
            self.scroll = round_to_line(local_top + self.scroll);
            self.bar.slider.set_pos(self.scroll);
        }
    }

    pub fn edit_var_at(&self, x: i32, y: i32) -> Option<&str> {
        let content = content_rect();
        if !content.contains(x, y) {
            return None;
        }
        html::edit_at(&self.lines, x - content.x, y - content.y + self.scroll)
    }

    pub fn button_at(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        let content = content_rect();
        if !content.contains(x, y) {
            return None;
        }
        let (px, py) = (x - content.x, y - content.y + self.scroll);
        self.lines.iter().find_map(|line| {
            line.widgets.iter().find_map(|widget| {
                let inside = px >= widget.x && px < widget.x + widget.width && py >= line.y && py < line.y + widget.height;
                (matches!(widget.kind, WidgetKind::Button { .. }) && inside).then_some((widget.x, line.y))
            })
        })
    }

    pub fn link_at(&self, x: i32, y: i32, font: &BitmapFont) -> Option<&str> {
        let content = content_rect();
        if !content.contains(x, y) {
            return None;
        }
        html::link_at(&self.lines, x - content.x, y - content.y + self.scroll, font)
    }

    pub fn link_message_at(&self, x: i32, y: i32) -> Option<&str> {
        let content = content_rect();
        if !content.contains(x, y) {
            return None;
        }
        html::link_message_at(&self.lines, x - content.x, y - content.y + self.scroll)
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        let (w, h) = NPC_VIEWER_SIZE;
        FrameCtrl::titled(&self.title, true).paint(list, x, y, w, h, font, alpha);
        list.draw_texture(x, y + TITLE_HEIGHT, w, h - TITLE_HEIGHT, 0, 0, w, h - TITLE_HEIGHT, NPC_HTML_BACK, alpha);
        let content = content_rect();
        let pushed = self.pressed.filter(|origin| self.button_at(self.pointer.map_or(-1, |p| p.0), self.pointer.map_or(-1, |p| p.1)) == Some(*origin));
        let start = list.quads.len();
        list.text_anim = self.anim_on.get().then(|| {
            let begin = self.anim_start_ms.get().unwrap_or_else(|| {
                self.anim_start_ms.set(Some(self.clock_ms.get()));
                self.clock_ms.get()
            });
            crate::draw::TextAnim::new(TEXT_ANIM_STEP, self.clock_ms.get().saturating_sub(begin) as f32 / 1000.0)
        });
        paint_html_nodes(list, x + content.x, y + content.y, (content.w, content.h), &self.lines, self.scroll, font, alpha, true, pushed);
        if list.text_anim.take().is_some_and(|anim| anim.finished) {
            self.anim_on.set(false);
            self.anim_start_ms.set(None);
        }
        list.clip_from(start, Rect::new(x + content.x, y + content.y, content.w, content.h));
        crate::html_form::paint_form(list, (x + content.x, y + content.y), (content.w, content.h), &self.lines, self.scroll, &self.form, font);
        self.form.paint_combos(list, (x + content.x, y + content.y), (content.w, content.h), &self.lines, self.scroll, font);
        draw_scroll_bar(list, x + content.x, y + content.y, (content.w, content.h), self.height, &self.bar);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn paint_html_viewer(
    list: &mut DrawList,
    cx: i32,
    cy: i32,
    size: (i32, i32),
    lines: &[Line],
    scroll: i32,
    font: &BitmapFont,
    alpha: f32,
) {
    let start = list.quads.len();
    paint_html_nodes(list, cx, cy, size, lines, scroll, font, alpha, true, None);
    list.clip_from(start, Rect::new(cx, cy, size.0, size.1));
    let content_height = html::height(lines, font);
    let mut bar = crate::scroll::ScrollWnd::new(size.0, size.1);
    bar.slider.set_range(content_height, size.1);
    bar.slider.set_pos(scroll);
    draw_scroll_bar(list, cx, cy, size, content_height, &bar);
}

fn draw_scroll_bar(list: &mut DrawList, cx: i32, cy: i32, size: (i32, i32), content_height: i32, bar: &crate::scroll::ScrollWnd) {
    if content_height > size.1 {
        bar.paint(list, cx, cy, size.0, size.1);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn paint_html_lines(
    list: &mut DrawList,
    cx: i32,
    cy: i32,
    size: (i32, i32),
    lines: &[Line],
    scroll: i32,
    font: &BitmapFont,
    alpha: f32,
) {
    paint_html_nodes(list, cx, cy, size, lines, scroll, font, alpha, false, None);
}

#[allow(clippy::too_many_arguments)]
fn paint_html_nodes(
    list: &mut DrawList,
    cx: i32,
    cy: i32,
    size: (i32, i32),
    lines: &[Line],
    scroll: i32,
    font: &BitmapFont,
    alpha: f32,
    partial: bool,
    pushed: Option<(i32, i32)>,
) {
    let content = Rect::new(0, 0, size.0, size.1);
    let step = html::line_step(font);
    for line in lines {
        for fill in &line.fills {
            let (top, bottom) = (fill.y - scroll, fill.y - scroll + fill.h);
            if bottom <= 0 || top >= content.h || fill.x >= content.w {
                continue;
            }
            list.draw_fill(cx + fill.x, cy + top, fill.w.min(content.w - fill.x), fill.h, fill.color);
        }
        for rule in &line.rules {
            if rule.x0 >= content.w {
                continue;
            }
            let y0 = rule.y0 - scroll;
            if rule.y0 == rule.y1 {
                if y0 >= 0 && y0 < content.h {
                    list.draw_hline(cx + rule.x0, cy + y0, rule.x1 - rule.x0 + 1, rule.color);
                }
            } else {
                list.draw_vline(cx + rule.x0, cy + y0, rule.y1 - rule.y0 + 1, rule.color);
            }
        }
        let top = line.y - scroll;
        for widget in &line.widgets {
            let outside = if partial { top + widget.height <= 0 || top >= content.h } else { top < 0 || top + widget.height > content.h };
            if outside || widget.x >= content.w {
                continue;
            }
            let width = widget.width.min(content.w - widget.x);
            match &widget.kind {
                WidgetKind::Edit { .. } | WidgetKind::ComboBox { .. } | WidgetKind::Link => {}
                WidgetKind::Image(src) => {
                    list.draw_texture(
                        cx + widget.x,
                        cy + top,
                        width,
                        widget.height,
                        0,
                        0,
                        width,
                        widget.height,
                        src,
                        alpha,
                    );
                }
                WidgetKind::Button { value, fore, back } => {
                    let face = if pushed == Some((widget.x, line.y)) { back } else { fore };
                    let full = widget.width;
                    list.draw_texture(cx + widget.x, cy + top, full, widget.height, 0, 0, full, widget.height, face, alpha);
                    let text_w = font.measure(value) as i32;
                    let tx = cx + widget.x + full / 2 - text_w / 2;
                    let ty = cy + top + widget.height / 2 - font.line_height as i32 / 2;
                    let anim = list.text_anim.take();
                    list.draw_text(tx, ty, HTML_BUTTON_TEXT, value, font);
                    list.text_anim = anim;
                }
            }
        }
        let hidden = if partial { top + step <= 0 || top >= content.h } else { top < 0 || top + step > content.h };
        if line.spans.is_empty() || hidden {
            continue;
        }
        for span in &line.spans {
            if span.x >= content.w {
                continue;
            }
            list.draw_text_clipped(cx + span.x, cy + top, content.w - span.x, span.color, &span.text, font);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::TEXT;
    use crate::draw::{LINE_TEXTURE, Quad};
    use crate::font::Glyphs;
    use crate::frame::{FRAME_BACK_LEFT, FRAME_CLOSE_BTN};
    use crate::html::LINK_COLOR;

    const SMALL: &[u8] = include_bytes!("../tests/fixtures/smallfont-e.gly");
    const MERCHANT: &str = include_str!("../tests/fixtures/npc-7829.htm");

    fn font() -> BitmapFont {
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(SMALL).unwrap())
    }

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    #[test]
    fn the_viewer_is_310_by_401_with_the_content_at_7_30() {
        assert_eq!(NPC_VIEWER_SIZE, (310, 401));
        assert_eq!(content_size(), (296, 364));
        assert_eq!(content_rect(), Rect::new(7, 30, 296, 364));
        assert_eq!(close_button(), Rect::new(287, 3, 15, 15));
        assert_eq!(grip(), Rect::new(0, 0, 310, 20));
    }

    #[test]
    fn paint_draws_the_title_frame_the_back_and_the_underlined_links() {
        let font = font();
        let viewer = NpcViewer::new("Chat", MERCHANT, &font);
        let mut list = DrawList::default();
        viewer.paint(&mut list, 100, 50, &font, 0.5);
        assert_eq!(list.quads[0].texture, FRAME_BACK_LEFT);
        assert_eq!((list.quads[0].x, list.quads[0].y), (100.0, 50.0));
        let back = with_texture(&list.quads, NPC_HTML_BACK);
        assert_eq!(back.len(), 1);
        assert_eq!((back[0].x, back[0].y, back[0].w, back[0].h), (100.0, 70.0, 310.0, 381.0));
        assert_eq!((back[0].u, back[0].v, back[0].uw, back[0].vh), (0.0, 0.0, 310.0, 381.0));
        assert_eq!(back[0].color, [1.0, 1.0, 1.0, 0.5]);
        assert_eq!(with_texture(&list.quads, FRAME_CLOSE_BTN).len(), 1);
        let text: Vec<&Quad> =
            list.quads.iter().filter(|q| q.texture == "L2Font-e.SmallFont-e" && q.y >= 70.0).collect();
        assert_eq!((text[0].x, text[0].y), (107.0, 80.0));
        assert_eq!(text[0].color, TEXT.rgba());
        let links: Vec<&Quad> = text.iter().copied().filter(|q| q.color == LINK_COLOR.rgba()).collect();
        assert!(!links.is_empty());
        let underlines = with_texture(&list.quads, LINE_TEXTURE);
        assert_eq!(underlines.len(), 5);
        let first_link_y = links[0].y;
        assert_eq!((underlines[0].x, underlines[0].y, underlines[0].h), (107.0, first_link_y + 13.0, 1.0));
        assert_eq!(underlines[0].color, LINK_COLOR.rgba());
        assert!(underlines[0].w > 30.0);
    }

    #[test]
    fn links_are_hit_in_window_coordinates_and_scroll_moves_them() {
        let font = font();
        let mut viewer = NpcViewer::new("Chat", MERCHANT, &font);
        assert_eq!(viewer.scroll, 0);
        assert_eq!(viewer.max_scroll(), 0);
        let sell = viewer.lines.iter().find(|l| l.spans[0].text == "Sell item").unwrap().y;
        assert_eq!(viewer.link_at(7 + 3, 30 + sell + 5, &font), Some("bypass -h npc_%objectId%_Sell"));
        assert_eq!(viewer.link_at(7 + 3, 30 + sell - 5, &font), None);
        assert_eq!(viewer.link_at(2, 30 + sell + 5, &font), None);
        assert_eq!(viewer.link_at(7 + 3, 10, &font), None);
        viewer.scroll_by(50);
        assert_eq!(viewer.scroll, 0);
        let tall = NpcViewer::new("Chat", &format!("<html><body>{}</body></html>", "line<br>".repeat(40)), &font);
        assert_eq!(tall.height, 40 * 28);
        let mut tall = tall;
        assert_eq!(tall.max_scroll(), 40 * 28 - 364);
        tall.scroll_by(100);
        assert_eq!(tall.scroll, 100);
        tall.scroll_by(-500);
        assert_eq!(tall.scroll, 0);
        tall.scroll_by(5000);
        assert_eq!(tall.scroll, tall.max_scroll());
        let mut list = DrawList::default();
        tall.paint(&mut list, 0, 0, &font, 1.0);
        let text: Vec<&Quad> =
            list.quads.iter().filter(|q| q.texture == "L2Font-e.SmallFont-e" && q.y >= 20.0).collect();
        assert!(text.iter().all(|q| q.y >= 30.0 && q.y + 13.0 <= 30.0 + 364.0));
        let last_line_top = 30.0 + (tall.height - 28 - tall.scroll) as f32;
        assert!(text.iter().any(|q| q.y == last_line_top));
    }

    #[test]
    fn images_and_buttons_are_painted_inside_the_content() {
        let font = font();
        let html = "<html><body><img src=\"L2UI.SquareWhite\" width=270 height=1><button value=\"Return\" action=\"bypass -h npc_1_Chat 0\" width=60 height=15 back=\"sek.cbui94\" fore=\"sek.cbui92\"></body></html>";
        let viewer = NpcViewer::new("Chat", html, &font);
        let mut list = DrawList::default();
        viewer.paint(&mut list, 0, 0, &font, 1.0);
        let image = with_texture(&list.quads, "L2UI.SquareWhite");
        assert_eq!((image[0].x, image[0].y, image[0].w, image[0].h, image[0].uw), (7.0, 30.0, 270.0, 1.0, 270.0));
        let button = with_texture(&list.quads, "sek.cbui92");
        assert_eq!((button[0].x, button[0].y, button[0].w, button[0].h), (7.0, 31.0, 60.0, 15.0));
        assert!(
            list.quads.iter().any(|q| q.texture == "L2Font-e.SmallFont-e"
                && q.color == HTML_BUTTON_TEXT.rgba()
                && q.y == 32.0)
        );
        assert_eq!(viewer.link_at(7 + 10, 30 + 1 + 7, &font), Some("bypass -h npc_1_Chat 0"));
    }

    const FORM: &str = "<html><body><edit var=\"name\" width=100 height=15 length=8 action=\"bypass -h npc_1_create $name\"><br><a action=\"bypass -h npc_1_create $name\">Create</a><br><a action=\"bypass -h npc_1_Buy 5\" msg=\"1983;Pet\">Buy</a></body></html>";

    #[test]
    fn the_edit_box_takes_focus_by_click_limits_its_length_and_fills_the_dollar_variable_of_the_bypass() {
        let font = font();
        let mut viewer = NpcViewer::new("Chat", FORM, &font);
        let edit = viewer.edit_var_at(7 + 3, 30 + 3).map(str::to_string);
        assert_eq!(edit.as_deref(), Some("name"));
        viewer.form.focus_var("name");
        for c in "abcdefghij".chars() {
            viewer.form.key(crate::chat::InputKey::Char(c), &font);
        }
        assert_eq!(viewer.form.fields[0].text(), "abcdefgh");
        assert_eq!(viewer.form.focused_action(), Some("bypass -h npc_1_create $name"));
        let (action, messages) = crate::html_form::expand_action("bypass -h npc_1_create $name", &viewer.form, &font);
        assert_eq!((action.as_str(), messages.len()), ("bypass -h npc_1_create abcdefgh", 0));
        let mut empty = NpcViewer::new("Chat", FORM, &font);
        empty.form.focus_var("name");
        let (action, messages) = crate::html_form::expand_action("bypass -h npc_1_create $name", &empty.form, &font);
        assert_eq!(action, "bypass -h npc_1_create");
        assert_eq!(messages, vec![crate::html_form::EMPTY_EDIT_SYSMSG]);
        let mut list = DrawList::default();
        viewer.paint(&mut list, 0, 0, &font, 1.0);
        assert!(!with_texture(&list.quads, crate::html_form::EDIT_TEXTURES[0]).is_empty());
    }

    #[test]
    fn a_link_with_msg_carries_its_message_in_front_of_the_action() {
        let font = font();
        let viewer = NpcViewer::new("Chat", FORM, &font);
        let buy = viewer.lines.iter().find(|line| line.spans.iter().any(|span| span.text == "Buy")).unwrap().y;
        let action = viewer.link_at(7 + 3, 30 + buy + 5, &font).unwrap();
        assert_eq!(html::split_message(action), (None, "bypass -h npc_1_Buy 5"));
        assert_eq!(viewer.link_message_at(7 + 3, 30 + buy + 5), Some("1983;Pet"));
        assert_eq!(html::message_code("1983;Pet"), Some((1983, "Pet".to_string())));
        assert_eq!(html::message_code("1983"), Some((1983, String::new())));
        assert_eq!(html::message_code("Really?"), None);
        assert_eq!(html::bypass_command(action).as_deref(), Some("npc_1_Buy 5"));
    }

    #[test]
    fn the_scroll_bar_buttons_step_one_line_and_the_thumb_drags_the_page() {
        let font = font();
        let mut tall = NpcViewer::new("Chat", &format!("<html><body>{}</body></html>", "line<br>".repeat(40)), &font);
        let content = content_rect();
        let rects = crate::scroll::scroll_rects(content.w, content.h);
        let down = (content.x + rects.down.x + 2, content.y + rects.down.y + 2);
        let up = (content.x + rects.up.x + 2, content.y + rects.up.y + 2);
        assert!(tall.scroll_press(down.0, down.1));
        assert_eq!(tall.scroll, LINE_STEP);
        assert!(tall.bar.down_pressed);
        tall.scroll_release();
        assert!(!tall.bar.down_pressed);
        assert!(tall.scroll_press(up.0, up.1));
        assert_eq!(tall.scroll, 0);
        tall.scroll_release();
        let thumb = (content.x + rects.slider.x + 2, content.y + rects.slider.y + 3);
        assert!(tall.scroll_press(thumb.0, thumb.1));
        assert!(tall.bar.slider.dragging);
        assert!(tall.scroll_move(thumb.1 + 40));
        assert!(tall.scroll > 0);
        tall.scroll_release();
        assert!(!tall.bar.slider.dragging);
        let mut short = NpcViewer::new("Chat", MERCHANT, &font);
        assert!(!short.scroll_press(down.0, down.1));
    }

    #[test]
    fn a_held_html_button_shows_the_back_face_only_while_the_pointer_stays_on_it() {
        let font = font();
        let html = "<html><body><button value=\"Return\" action=\"bypass -h npc_1_Chat 0\" width=60 height=15 back=\"sek.cbui94\" fore=\"sek.cbui92\"></body></html>";
        let mut viewer = NpcViewer::new("Chat", html, &font);
        let inside = (7 + 10, 30 + 1 + 7);
        viewer.pointer = Some(inside);
        viewer.pressed = viewer.button_at(inside.0, inside.1);
        assert!(viewer.pressed.is_some());
        let mut list = DrawList::default();
        viewer.paint(&mut list, 0, 0, &font, 1.0);
        assert_eq!(with_texture(&list.quads, "sek.cbui94").len(), 1);
        assert!(with_texture(&list.quads, "sek.cbui92").is_empty());
        viewer.pointer = Some((300, 300));
        let mut list = DrawList::default();
        viewer.paint(&mut list, 0, 0, &font, 1.0);
        assert!(with_texture(&list.quads, "sek.cbui94").is_empty());
        assert_eq!(with_texture(&list.quads, "sek.cbui92").len(), 1);
    }

    #[test]
    fn the_scroll_bar_value_is_rounded_up_to_a_whole_line_and_an_edit_above_the_top_scrolls_into_view() {
        assert_eq!((round_to_line(0), round_to_line(1), round_to_line(13), round_to_line(14)), (0, 13, 13, 26));
        let font = font();
        let html = format!("<html><body>{}<edit var=\"Name\" width=100 height=15><br>{}</body></html>", "x<br>".repeat(8), "y<br>".repeat(40));
        let mut viewer = NpcViewer::new("Chat", &html, &font);
        let top = viewer.lines.iter().find(|line| line.widgets.iter().any(|w| matches!(w.kind, WidgetKind::Edit { .. }))).map(|line| line.y).unwrap_or(0);
        viewer.scroll_by(top + 5);
        let local = top - viewer.scroll;
        assert!(local < 0);
        viewer.scroll_to_edit(local);
        assert_eq!(viewer.scroll, round_to_line(top));
        assert_eq!(viewer.bar.slider.pos, viewer.scroll);
        viewer.scroll_to_edit(5);
        assert_eq!(viewer.scroll, round_to_line(top));
    }

    #[test]
    fn the_combo_box_opens_picks_a_row_paints_the_list_and_fills_the_dollar_variable() {
        let font = font();
        let html = "<html><body><combobox var=\"kind\" list=\"Sword;Bow;Staff\" sel=\"Bow\" width=80><br><br><br><br><br><a action=\"bypass -h npc_1_pick $kind\">Go</a></body></html>";
        let mut viewer = NpcViewer::new("Chat", html, &font);
        assert_eq!(viewer.form.combos.len(), 1);
        assert_eq!((viewer.form.combos[0].selected, viewer.form.combos[0].entries.len()), (1, 3));
        let (px, py) = (7 + 10, 30 + 5);
        assert!(viewer.combo_press(px, py));
        assert!(viewer.form.combos[0].open);
        let mut list = DrawList::default();
        viewer.paint(&mut list, 0, 0, &font, 1.0);
        assert_eq!(with_texture(&list.quads, crate::html_form::EDIT_TEXTURES[1]).len(), 5);
        viewer.combo_move(px, 30 + 17 * 3 + 5);
        assert_eq!(viewer.form.combos[0].hover, 2);
        let mut list = DrawList::default();
        viewer.paint(&mut list, 0, 0, &font, 1.0);
        assert!(!with_texture(&list.quads, crate::html_form::COMBO_SELECT).is_empty());
        assert!(viewer.combo_press(px, 30 + 17 * 3 + 5));
        assert_eq!((viewer.form.combos[0].selected, viewer.form.combos[0].open), (2, false));
        let (action, messages) = crate::html_form::expand_action("bypass -h npc_1_pick $kind", &viewer.form, &font);
        assert_eq!((action.as_str(), messages.len()), ("bypass -h npc_1_pick Staff", 0));
        assert!(!viewer.combo_press(300, 300));
        assert!(viewer.combo_press(px, py));
        assert!(viewer.combo_press(300, 300));
        assert!(!viewer.form.combos[0].open);
        assert_eq!(crate::html_form::FormCombo::from_list("c", "A;B", "").selected, 0);
        assert_eq!(crate::html_form::FormCombo::from_list("c", "A;B", "Z").selected, -1);
    }

    #[test]
    fn the_combo_arrow_shows_the_over_and_pushed_faces_and_opens_the_list_on_release() {
        let font = font();
        let html = "<html><body><combobox var=\"kind\" list=\"Sword;Bow;Staff\" sel=\"Bow\" width=80></body></html>";
        let mut viewer = NpcViewer::new("Chat", html, &font);
        let arrow = |viewer: &NpcViewer| {
            let mut list = DrawList::default();
            viewer.paint(&mut list, 0, 0, &font, 1.0);
            [crate::html_form::COMBO_ARROW, crate::html_form::COMBO_ARROW_OVER, crate::html_form::COMBO_ARROW_DOWN]
                .into_iter()
                .map(|name| with_texture(&list.quads, name).len())
                .collect::<Vec<_>>()
        };
        assert_eq!(arrow(&viewer), [1, 0, 0]);
        let (ax, ay) = (7 + 80 - 16 + 3, 30 + 1 + 3);
        viewer.combo_move(ax, ay);
        assert_eq!(arrow(&viewer), [0, 1, 0]);
        assert!(viewer.combo_press(ax, ay));
        assert!(!viewer.form.combos[0].open);
        assert_eq!(arrow(&viewer), [0, 0, 1]);
        viewer.combo_move(7 + 10, 30 + 5);
        assert_eq!(arrow(&viewer), [1, 0, 0]);
        viewer.combo_move(ax, ay);
        assert!(viewer.combo_release(ax, ay));
        assert!(viewer.form.combos[0].open);
        assert!(viewer.combo_press(ax, ay));
        assert!(viewer.form.combos[0].open);
        viewer.combo_release(7 + 10, 30 + 5);
        assert!(viewer.form.combos[0].open);
        let mut away = NpcViewer::new("Chat", html, &font);
        away.combo_press(ax, ay);
        away.combo_release(7 + 10, 30 + 5);
        assert!(!away.form.combos[0].open);
    }

    #[test]
    fn a_page_with_textani_one_reveals_a_glyph_every_seven_milliseconds_and_stops_with_a_click() {
        let font = font();
        let html = "<html textani=\"1\"><body>abcdefghij</body></html>";
        let viewer = NpcViewer::new("Chat", html, &font);
        assert!(viewer.anim_on.get());
        let glyphs = |viewer: &NpcViewer| {
            let mut list = DrawList::default();
            viewer.paint(&mut list, 0, 0, &font, 1.0);
            list.quads.iter().filter(|q| q.texture == "L2Font-e.SmallFont-e" && q.y >= 30.0).count()
        };
        viewer.set_clock_ms(1000);
        assert_eq!(glyphs(&viewer), 1);
        viewer.set_clock_ms(1000 + 22);
        assert_eq!(glyphs(&viewer), 4);
        viewer.set_clock_ms(1000 + 7 * 100);
        assert_eq!(glyphs(&viewer), 10);
        assert!(!viewer.anim_on.get());
        let other = NpcViewer::new("Chat", "<html textani=\"1\"><body>abc</body></html>", &font);
        other.set_clock_ms(5);
        glyphs(&other);
        other.stop_text_anim();
        assert_eq!(glyphs(&other), 3);
        assert!(!NpcViewer::new("Chat", "<html textani=\"2\"><body>abc</body></html>", &font).anim_on.get());
        assert!(!NpcViewer::new("Chat", "<html><body>abc</body></html>", &font).anim_on.get());
    }
}
