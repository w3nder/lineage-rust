use killer_ui::html::{Cell, Node};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "t", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum HtmlNode {
    Text { text: String },
    Br,
    Font { color: Option<String>, children: Vec<HtmlNode> },
    Link { action: Option<String>, children: Vec<HtmlNode> },
    Center { children: Vec<HtmlNode> },
    Table { width: Option<i32>, rows: Vec<Vec<HtmlCell>> },
    Img { src: String, width: i32, height: i32 },
    Button { value: String, action: Option<String>, width: i32, height: i32 },
    Edit { var: String, width: i32, height: i32, multi: bool },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HtmlCell {
    pub width: Option<i32>,
    pub children: Vec<HtmlNode>,
}

fn known(action: &str) -> bool {
    killer_ui::html::bypass_command(action).is_some() || killer_ui::html::link_target(action).is_some() || killer_ui::html::cmd_target(action).is_some()
}

fn action(raw: &str, object_id: i32) -> Option<String> {
    known(raw).then(|| raw.replace("%objectId%", &object_id.to_string()))
}

fn hex(argb: u32) -> String {
    format!("#{:06x}", argb & 0x00ff_ffff)
}

pub fn convert(nodes: &[Node], object_id: i32) -> Vec<HtmlNode> {
    nodes.iter().map(|node| one(node, object_id)).collect()
}

fn one(node: &Node, object_id: i32) -> HtmlNode {
    match node {
        Node::Text(text) => HtmlNode::Text { text: text.clone() },
        Node::Br | Node::Br1 => HtmlNode::Br,
        Node::Font { color, children } => HtmlNode::Font { color: color.map(|argb| hex(argb.0)), children: convert(children, object_id) },
        Node::Link { action: raw, children } => HtmlNode::Link { action: action(raw, object_id), children: convert(children, object_id) },
        Node::Center(children) => HtmlNode::Center { children: convert(children, object_id) },
        Node::Table { width, rows } => HtmlNode::Table { width: *width, rows: rows.iter().map(|row| row.iter().map(|cell: &Cell| HtmlCell { width: cell.width, children: convert(&cell.children, object_id) }).collect()).collect() },
        Node::Img { src, width, height } => HtmlNode::Img { src: src.clone(), width: *width, height: *height },
        Node::Button { value, action: raw, width, height, .. } => HtmlNode::Button { value: value.clone(), action: action(raw, object_id), width: *width, height: *height },
        Node::Edit { var, width, height, multi } => HtmlNode::Edit { var: var.clone(), width: *width, height: *height, multi: *multi },
    }
}

pub fn user_command(action: &str) -> Option<killer_bot_core::command::UserCommand> {
    use killer_bot_core::chat::Channel;
    use killer_bot_core::command::UserCommand;
    if let Some(text) = killer_ui::html::cmd_target(action) {
        return Some(UserCommand::Say { channel: Channel::All, target: String::new(), text: text.to_owned() });
    }
    if let Some(command) = killer_ui::html::bypass_command(action) {
        return Some(UserCommand::Bypass(command));
    }
    killer_ui::html::link_target(action).map(UserCommand::LinkHtml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_actions_get_the_object_id_and_unknown_ones_are_dropped() {
        let document = killer_ui::html::parse("<html><body><a action=\"bypass -h npc_%objectId%_Chat 1\">Ir</a><a action=\"nada\">x</a><button value=\"Fechar\" action=\"link a.htm\" width=60 height=15></body></html>");
        let nodes = convert(&document.body, 77);
        let actions: Vec<Option<String>> = nodes
            .iter()
            .filter_map(|node| match node {
                HtmlNode::Link { action, .. } | HtmlNode::Button { action, .. } => Some(action.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(actions, vec![Some("bypass -h npc_77_Chat 1".to_owned()), None, Some("link a.htm".to_owned())]);
    }

    #[test]
    fn actions_map_to_the_three_commands_of_the_desktop() {
        use killer_bot_core::command::UserCommand;
        assert_eq!(user_command("bypass -h npc_1_Chat 1"), Some(UserCommand::Bypass("npc_1_Chat 1".to_owned())));
        assert_eq!(user_command("link teleport.htm"), Some(UserCommand::LinkHtml("teleport.htm".to_owned())));
        assert!(matches!(user_command("\u{1}cmd:/loc"), Some(UserCommand::Say { .. })));
        assert_eq!(user_command("nada"), None);
    }

    #[test]
    fn colors_become_css_hex() {
        assert_eq!(hex(0xff_ff_cc_00), "#ffcc00");
    }
}
