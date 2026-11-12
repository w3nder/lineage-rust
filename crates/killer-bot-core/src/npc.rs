use killer_ui::html::{Node, bypass_command, parse};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NpcDialog {
    pub npc_object_id: i32,
    pub text: String,
    pub links: Vec<NpcLink>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpcLink {
    pub label: String,
    pub command: String,
}

fn plain(nodes: &[Node], out: &mut String) {
    for node in nodes {
        match node {
            Node::Text(text) => out.push_str(text),
            Node::Br | Node::Br1 => out.push('\n'),
            Node::Font { children, .. } | Node::Center(children) => plain(children, out),
            Node::Link { .. } => {}
            Node::Table { rows, .. } => {
                for row in rows {
                    for cell in row {
                        plain(&cell.children, out);
                        out.push(' ');
                    }
                    out.push('\n');
                }
            }
            Node::Img { .. } | Node::Button { .. } | Node::Edit { .. } => {}
        }
    }
}

fn links(nodes: &[Node], npc_object_id: i32, out: &mut Vec<NpcLink>) {
    for node in nodes {
        match node {
            Node::Link { action, children } => {
                if let Some(command) = bypass_command(action) {
                    let mut label = String::new();
                    plain(children, &mut label);
                    out.push(NpcLink { label: label.trim().to_owned(), command: command.replace("%objectId%", &npc_object_id.to_string()) });
                }
            }
            Node::Button { value, action, .. } => {
                if let Some(command) = bypass_command(action) {
                    out.push(NpcLink { label: value.clone(), command: command.replace("%objectId%", &npc_object_id.to_string()) });
                }
            }
            Node::Font { children, .. } | Node::Center(children) => links(children, npc_object_id, out),
            Node::Table { rows, .. } => {
                for row in rows {
                    for cell in row {
                        links(&cell.children, npc_object_id, out);
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn parse_npc_html(npc_object_id: i32, html: &str) -> NpcDialog {
    let document = parse(html);
    let mut text = String::new();
    plain(&document.body, &mut text);
    let mut found = Vec::new();
    links(&document.body, npc_object_id, &mut found);
    let text = text.lines().map(str::trim).collect::<Vec<_>>().join("\n").trim().to_owned();
    NpcDialog { npc_object_id, text, links: found }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_links_with_bypass_actions() {
        let dialog = parse_npc_html(
            77,
            "<html><body>Gatekeeper:<br>Where to?<br><a action=\"bypass -h npc_%objectId%_Chat 1\">Teleport</a><br><a action=\"bypass -h npc_%objectId%_Quest\">Quest</a></body></html>",
        );
        assert!(dialog.text.contains("Where to?"));
        assert_eq!(
            dialog.links,
            vec![
                NpcLink { label: "Teleport".into(), command: "npc_77_Chat 1".into() },
                NpcLink { label: "Quest".into(), command: "npc_77_Quest".into() },
            ]
        );
    }

    #[test]
    fn plain_text_without_links_keeps_the_text() {
        let dialog = parse_npc_html(1, "Hello adventurer");
        assert_eq!(dialog.text, "Hello adventurer");
        assert!(dialog.links.is_empty());
    }

    #[test]
    fn malformed_html_does_not_panic_and_keeps_readable_text() {
        for html in ["<html><body><a action=\"bypass -h x\">sem fim", "<<<>>>", "<a", "", "</a></b><br1><font color=>oi"] {
            let dialog = parse_npc_html(1, html);
            let _ = dialog.text.len();
        }
        assert!(parse_npc_html(1, "<b>texto<").text.contains("texto"));
    }

    #[test]
    fn button_actions_become_links() {
        let dialog = parse_npc_html(5, "<button value=\"Buy\" action=\"bypass -h npc_%objectId%_Buy 92\" width=60 height=15 back=\"a\" fore=\"b\">");
        assert_eq!(dialog.links, vec![NpcLink { label: "Buy".into(), command: "npc_5_Buy 92".into() }]);
    }
}
