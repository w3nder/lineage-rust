use killer_ui::font::{BitmapFont, Glyphs};
use killer_ui::html;
use killer_ui::html_layout::{self, LeafKind};
use serde_json::{Value, json};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn real_font() -> BitmapFont {
    BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(include_bytes!("fixtures/smallfont-e.gly")).unwrap())
}

fn widths(font: &BitmapFont) -> Value {
    let table: serde_json::Map<String, Value> = (0x20u32..0x7f)
        .filter_map(|code| char::from_u32(code).and_then(|c| font.glyph(c)).map(|g| (code.to_string(), json!(g.width))))
        .collect();
    Value::Object(table)
}

fn oracle(cases: &[Value]) -> Option<Vec<Value>> {
    std::env::var_os("KILLER_CLIENT_DIR").map(PathBuf::from)?;
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scripts/html_reference.py");
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(script)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    write!(child.stdin.take().unwrap(), "{}", json!(cases)).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    Some(serde_json::from_slice(&output.stdout).unwrap())
}

fn leaf_type(kind: &LeafKind, is_text: bool) -> i64 {
    if is_text {
        1
    } else if matches!(kind, LeafKind::Image { .. }) {
        5
    } else {
        0
    }
}

fn native_child(child: &html_layout::Child, depth: usize, out: &mut Vec<String>) {
    let pad = " ".repeat(depth);
    match child {
        html_layout::Child::Leaf(leaf) => {
            let text = if leaf.is_text { leaf.text.clone() } else { String::new() };
            out.push(format!("{pad}leaf t={} x={} y={} w={} h={} f4={} f8={} {:?} {:?}", leaf_type(&leaf.kind, leaf.is_text), leaf.x, leaf.y, leaf.w, leaf.h, leaf.start, leaf.end, text, leaf.colors));
        }
        html_layout::Child::Container(container) => native_container(container, depth, out),
        html_layout::Child::Table(table) => {
            out.push(format!(
                "{pad}table x={} y={} w={} h={} min={} pad={} spacing={} border={} w54={} pct={} colw={:?} colpos={:?} rowh={:?}",
                table.x, table.y, table.w, table.h, table.min_w, table.padding, table.spacing, table.border, table.width_attr, table.percent, table.column_widths, table.column_positions, table.row_heights
            ));
            for row in &table.rows {
                out.push(format!("{pad} row"));
                for cell in row {
                    native_container(cell, depth + 2, out);
                }
            }
        }
    }
}

fn native_container(container: &html_layout::Container, depth: usize, out: &mut Vec<String>) {
    let pad = " ".repeat(depth);
    out.push(format!("{pad}cell x={} y={} w={} h={} min={} align={} valign={} yoff={}", container.x, container.y, container.w, container.h, container.min_w, container.align, container.valign, container.y_offset));
    for child in &container.children {
        native_child(child, depth + 1, out);
    }
}

fn native_tree(html_text: &str, width: i32, font: &BitmapFont) -> Vec<String> {
    let doc = html::parse(html_text);
    let Some(tree) = html_layout::layout_tree(doc.tokens(), width - 16, font) else { return Vec::new() };
    let mut out = Vec::new();
    for child in &tree.root.children {
        native_child(child, 0, &mut out);
    }
    out
}

fn original_node(node: &Value, depth: usize, out: &mut Vec<String>) {
    let pad = " ".repeat(depth);
    let int = |key: &str| node[key].as_i64().unwrap_or(0);
    match node["kind"].as_str().unwrap_or("") {
        "leaf" => {
            let ty = int("type");
            let text = if ty == 1 { node["f0"].as_str().unwrap_or("").to_string() } else { String::new() };
            let colors = if ty == 1 { node["e4"].as_str().unwrap_or("").replace("\\x1e", "\u{1e}") } else { String::new() };
            out.push(format!("{pad}leaf t={} x={} y={} w={} h={} f4={} f8={} {:?} {:?}", ty, int("x"), int("y"), int("w"), int("h"), int("f4"), int("f8"), text, colors));
        }
        "container" => original_container(node, depth, out),
        "table" => {
            let list = |key: &str, count: usize| -> Vec<i64> { node[key].as_array().map(|a| a.iter().take(count).map(|v| v.as_i64().unwrap()).collect()).unwrap_or_default() };
            let rows = node["rows"].as_array().unwrap();
            let columns = rows.iter().map(|r| r.as_array().unwrap().len()).max().unwrap_or(0);
            out.push(format!(
                "{pad}table x={} y={} w={} h={} min={} pad={} spacing={} border={} w54={} pct={} colw={:?} colpos={:?} rowh={:?}",
                int("x"), int("y"), int("w"), int("h"), int("min"), int("pad"), int("spacing"), int("border"), int("w54"), int("w58") != 0, list("col_w", columns), list("col_pos", columns), list("row_h", rows.len())
            ));
            for row in rows {
                out.push(format!("{pad} row"));
                for cell in row.as_array().unwrap() {
                    original_container(cell, depth + 2, out);
                }
            }
        }
        other => out.push(format!("{pad}?{other}")),
    }
}

fn original_container(node: &Value, depth: usize, out: &mut Vec<String>) {
    let pad = " ".repeat(depth);
    let int = |key: &str| node[key].as_i64().unwrap_or(0);
    out.push(format!("{pad}cell x={} y={} w={} h={} min={} align={} valign={} yoff={}", int("x"), int("y"), int("w"), int("h"), int("min"), int("align"), int("valign"), int("yoff")));
    for child in node["children"].as_array().unwrap() {
        original_node(child, depth + 1, out);
    }
}

fn original_tree(result: &Value) -> Vec<String> {
    let mut out = Vec::new();
    for node in result["tree"].as_array().unwrap() {
        original_node(node, 0, &mut out);
    }
    out
}

fn native_paint(html_text: &str, width: i32, font: &BitmapFont) -> Vec<String> {
    use html_layout::Primitive;
    let doc = html::parse(html_text);
    let painted = html_layout::run(doc.tokens(), width - 16, font);
    let mut out = Vec::new();
    let mut rest = Vec::new();
    for primitive in &painted.primitives {
        match primitive {
            Primitive::Text(text) => {
                let segments: Vec<(i64, i64, u32)> = text.segments.iter().map(|(a, b, c)| (*a as i64, *b as i64, c.0)).collect();
                let letters: Vec<char> = text.text.chars().filter(|c| *c != ' ').collect();
                let glyphs: Vec<(i32, i32, char)> = text.glyphs.iter().filter(|(x, _, _, _)| *x < width).zip(&letters).map(|((x, y, _, _), c)| (*x, *y, *c)).collect();
                out.push(format!("T base={:08x} segs={:?} glyphs={:?}", text.base.0, segments, glyphs));
                for (x0, x1, y) in &text.underline {
                    rest.push(format!("L {x0} {y} {x1} {y} {:08x}", text.base.0));
                }
            }
            Primitive::Line { x0, y0, x1, y1 } => rest.push(format!("L {x0} {y0} {x1} {y1} ffdcdcdc")),
            Primitive::Fill { x, y, w, h, color } => rest.push(format!("R {x} {y} {w} {h} {color:08x}")),
            Primitive::Image { x, y, w, h, .. } => rest.push(format!("X {x} {y} {w} {h}")),
        }
    }
    out.extend(rest);
    for item in &painted.boxes {
        match &item.kind {
            LeafKind::Text => out.push(format!("W link {} {} {} {} wrap={} end={}", item.x, item.y, item.w, item.h, item.wrap, item.end)),
            LeafKind::Button { .. } => out.push(format!("W button {} {} {} {}", item.x, item.y, item.w, item.h)),
            LeafKind::Edit { .. } => out.push(format!("W edit {} {} {} {}", item.x, item.y, item.w, item.h)),
            LeafKind::MultiEdit { .. } => out.push(format!("W multiedit {} {} {} {}", item.x, item.y, item.w, item.h)),
            LeafKind::ComboBox { .. } => out.push(format!("W combobox {} {} {} {}", item.x, item.y, item.w, item.h)),
            _ => {}
        }
    }
    out
}

fn original_paint(result: &Value) -> Vec<String> {
    let mut out = Vec::new();
    for text in result["texts"].as_array().unwrap() {
        let string = text["text"].as_str().unwrap_or("");
        let letters: Vec<char> = string.chars().filter(|c| *c != ' ').collect();
        let glyphs: Vec<(i32, i32, char)> = text["glyphs"].as_array().unwrap().iter().zip(&letters).map(|(g, c)| (g[0].as_f64().unwrap() as i32, g[1].as_f64().unwrap() as i32, *c)).collect();
        let segments: Vec<(i64, i64, u32)> = text["segments"].as_array().unwrap().iter().map(|s| (s[0].as_i64().unwrap(), (s[1].as_i64().unwrap() as i32) as i64, s[2].as_u64().unwrap() as u32)).collect();
        out.push(format!("T base={:08x} segs={:?} glyphs={:?}", text["color"].as_u64().unwrap(), segments, glyphs));
    }
    for draw in result["draws"].as_array().unwrap() {
        let args: Vec<i64> = draw["args"].as_array().unwrap().iter().map(|v| v.as_i64().unwrap()).collect();
        match draw["call"].as_str().unwrap() {
            "UCanvas::DrawLine" => out.push(format!("L {} {} {} {} {:08x}", args[0], args[1], args[2], args[3], args[4] as u32)),
            "UCanvas::DrawRect" => out.push(format!("R {} {} {} {} {:08x}", args[0], args[1], args[2], args[3], args[5] as u32)),
            "UCanvas::DrawTexture" => out.push(format!("X {} {} {} {}", args[0], args[1], args[2], args[3])),
            _ => {}
        }
    }
    for widget in result["widgets"].as_array().unwrap() {
        let args: Vec<i64> = widget["args"].as_array().unwrap().iter().map(|v| v.as_i64().unwrap_or(0)).collect();
        let kind = widget["kind"].as_str().unwrap();
        match kind {
            "link1" | "link2" => out.push(format!("W link {} {} {} {} wrap={} end={}", args[0], args[1], args[4], args[5], args[6], args[7])),
            other => out.push(format!("W {other} {} {} {} {}", args[0], args[1], args[2], args[3])),
        }
    }
    out
}

fn compare(cases: &[String], width: i32) -> usize {
    let font = real_font();
    let requests: Vec<Value> = cases.iter().map(|case| json!({"html": case, "width": width, "height": 4000, "widths": widths(&font)})).collect();
    let Some(results) = oracle(&requests) else { return 0 };
    let mut failures = 0;
    let mut skipped = 0;
    for (case, result) in cases.iter().zip(&results) {
        if std::env::var("SKIP_NARROW").is_ok() {
            let doc = html::parse(case);
            let narrow = html_layout::layout_tree(doc.tokens(), width - 16, &font).is_some_and(|tree| tree.leaves().iter().any(|leaf| leaf.is_text && leaf.wrap_w < 45));
            if narrow {
                skipped += 1;
                continue;
            }
        }
        let (expected, actual) = (original_tree(result), native_tree(case, width, &font));
        if std::env::var("SHOW").is_ok() {
            eprintln!("=== {}\n{}", &case[..case.len().min(120)], expected.join("\n"));
        }
        let (paint_expected, paint_actual) = (original_paint(result), native_paint(case, width, &font));
        if std::env::var("FULL").is_ok() && (expected != actual || paint_expected != paint_actual) {
            std::fs::write(format!("/tmp/fail_{failures}.html"), case).unwrap();
        }
        if expected != actual {
            failures += 1;
            let at = expected.iter().zip(&actual).position(|(a, b)| a != b).unwrap_or(expected.len().min(actual.len()));
            eprintln!("--- ARVORE {}\n primeira diferenca na linha {at} (original {} linhas, porte {}):\n original: {}\n porte:    {}", &case[..case.len().min(120)].replace('\n', " "), expected.len(), actual.len(), expected.get(at).cloned().unwrap_or_default(), actual.get(at).cloned().unwrap_or_default());
        } else if paint_expected != paint_actual {
            failures += 1;
            let at = paint_expected.iter().zip(&paint_actual).position(|(a, b)| a != b).unwrap_or(paint_expected.len().min(paint_actual.len()));
            let show = |list: &Vec<String>| list.get(at).map(|line| line.chars().take(600).collect::<String>()).unwrap_or_default();
            eprintln!("--- PINTURA {}\n primeira diferenca no item {at} (original {} itens, porte {}):\n original: {}\n porte:    {}", &case[..case.len().min(120)].replace('\n', " "), paint_expected.len(), paint_actual.len(), show(&paint_expected), show(&paint_actual));
        }
    }
    if skipped > 0 {
        eprintln!("{skipped} casos estreitos ignorados");
    }
    failures
}

fn wrap(body: &str) -> String {
    format!("<html><body>{body}</body></html>")
}

#[test]
#[ignore]
fn leaf_tree_matches_the_original_for_text_documents() {
    let bodies = [
        "Hello world",
        "Hello world<br>Second line<br><br>Fourth",
        "This is the<font color=\"LEVEL\"> dueling area</font>.",
        "A <font color=\"LEVEL\">x  y </font> B",
        "<font color=\"LEVEL\">Level</font> Level",
        "<FONT color=\"LEVEL\">upper</FONT> tags",
        "a<br/>b",
        "a <a action=\"x\">one<br>two</a> end",
        "  lead  <b>bold</b>  trail  ",
        "x<font color=red>r</font><font color=blue>b</font>y",
        "m<font>noattr</font>n",
        "A<br1>B<br1>C",
        "<center>Centered</center>After",
        "x <a action=\"bypass -h a\">link text that is long enough to wrap around the viewer width for sure</a> tail text that follows the link",
        "<img src=\"L2UI.SquareWhite\" width=270 height=1>after",
        "text<img src=\"a.b\" width=20 height=30>more text<img src=\"c.d\" height=5>",
        "<img>x<img width=abc height=-3>",
        "<button value=\"Go\" action=\"bypass -h b\" width=60 height=15 back=\"a\" fore=\"b\">",
        "<button>x<button width=5>",
        "<edit var=\"nm\" width=100 height=15>after<multiedit var=\"mm\" width=120 height=60>tail",
        "<edit>x<multiedit>y<combobox>z<combobox var=\"c\" list=\"a;b\" width=80>",
        "<center><button value=\"Go\" width=60 height=15><br></center>row two",
        "<center>mid</center><br>left<center>again</center>",
        "start<center>mid text</center>end text",
        "<a action=\"x\">link</a> plain <a cmd=\"say hi\">cmd</a> <a link=\"l.htm\">file</a> <a href=\"zzz\">nolink</a> end",
        "Intro<table><tr><td>One</td><td>Two</td></tr><tr><td>Three</td><td>Four</td></tr></table>after",
        "<table border=1><tr><td>a</td><td>b</td></tr></table>",
        "<table width=260><tr><td width=40>a</td><td>b</td></tr><tr><td fixwidth=60>c</td></tr></table>",
        "<table width=50%><tr><td>left cell text</td><td align=right>right</td></tr></table>",
        "<table cellpadding=4 cellspacing=6><tr><td>a</td><td valign=top>tall<br>cell<br>text</td><td valign=bottom>b</td></tr></table>",
        "<table><tr><td>one</td></tr></table><table><tr><td>two</td></tr></table>",
        "<table height=100><tr><td>a</td></tr><tr><td>b</td></tr></table>",
        "<center><table><tr><td>centered table</td></tr></table></center>",
        "<table><tr><td><table><tr><td>inner</td></tr></table></td><td>outer</td></tr></table>",
        "<table bgcolor=\"336699\" width=200><tr><td><a action=\"x\">link</a> text that wraps around inside narrow cells of this table</td></tr></table>",
        "Gatekeeper Roxxy:<br>I'll teleport you wherever you like, but it costs adena and sometimes more than you expect, so choose wisely before you go.<br><a action=\"bypass -h npc_1_Chat 1\">Teleport</a>",
    ];
    let cases: Vec<String> = bodies.iter().map(|body| wrap(body)).collect();
    assert_eq!(compare(&cases, 296), 0);
}


struct Random(u64);

impl Random {
    fn next(&mut self, below: u64) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) % below
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.next(items.len() as u64) as usize]
    }
}

fn words(random: &mut Random) -> String {
    const WORDS: [&str; 14] = ["a", "to", "the", "Gatekeeper", "Roxxy:", "I'll", "teleport", "you", "wherever", "supercalifragilistic", "W", "mmmm", "adena", "7,000"];
    let count = 1 + random.next(9);
    let mut text = String::new();
    for index in 0..count {
        if index > 0 {
            text.push_str(random.pick(&[" ", " ", " ", "  ", "\n"]));
        }
        text.push_str(random.pick(&WORDS));
    }
    text
}

fn inline(random: &mut Random, depth: u32) -> String {
    let mut out = String::new();
    for _ in 0..1 + random.next(4) {
        match random.next(14) {
            0..=3 => out.push_str(&words(random)),
            4 => out.push_str("<br>"),
            5 => out.push_str("<br1>"),
            6 => out.push_str(&format!("<font color=\"{}\">{}</font>", random.pick(&["LEVEL", "FF0000", "00ffff", "abc"]), words(random))),
            7 => out.push_str(&format!("<a action=\"bypass -h {}\">{}</a>", random.pick(&["a", "npc_1_Chat 1"]), words(random))),
            8 => out.push_str(&format!("<center>{}</center>", words(random))),
            9 => out.push_str(&format!("<img src=\"x.y\" width={} height={}>", 1 + random.next(120), random.next(30))),
            10 => out.push_str(&format!("<button value=\"Go\" width={} height={}>", 20 + random.next(80), 10 + random.next(20))),
            11 => out.push_str(&format!("<edit var=\"v\" width={} height={}>", 20 + random.next(100), 10 + random.next(10))),
            _ if depth < 2 => out.push_str(&table(random, depth + 1)),
            _ => out.push_str(&words(random)),
        }
    }
    out
}

fn table(random: &mut Random, depth: u32) -> String {
    let mut attrs = String::new();
    for (name, values) in [("border", vec!["0", "1", "2"]), ("width", vec!["120", "200", "270", "50%", "100%"]), ("cellpadding", vec!["0", "2", "4"]), ("cellspacing", vec!["0", "3"]), ("height", vec!["60", "120"]), ("bgcolor", vec!["336699"])] {
        if random.next(4) == 0 {
            attrs.push_str(&format!(" {name}={}", random.pick(&values)));
        }
    }
    let mut out = format!("<table{attrs}>");
    for _ in 0..1 + random.next(3) {
        out.push_str("<tr>");
        for _ in 0..1 + random.next(3) {
            let mut cell = String::new();
            for (name, values) in [("width", vec!["30", "60", "40%"]), ("align", vec!["center", "right", "left"]), ("valign", vec!["top", "bottom", "center"]), ("height", vec!["40"]), ("fixwidth", vec!["70"])] {
                if random.next(5) == 0 {
                    cell.push_str(&format!(" {name}={}", random.pick(&values)));
                }
            }
            out.push_str(&format!("<td{cell}>{}</td>", inline(random, depth)));
        }
        out.push_str("</tr>");
    }
    out.push_str("</table>");
    out
}

const FIXTURES: &[&str] = &[
    include_str!("fixtures/npc-7829.htm"),
    include_str!("fixtures/npc-agitsale1.htm"),
    include_str!("fixtures/npc-dawn-priest-2c.htm"),
    include_str!("fixtures/npc-heine-7899-1.htm"),
    include_str!("fixtures/npc-heine-8067.htm"),
    include_str!("fixtures/npc-signboard.htm"),
    include_str!("fixtures/npc-territorystatus.htm"),
];

#[test]
#[ignore]
fn real_npc_pages_match_the_original() {
    let cases: Vec<String> = FIXTURES.iter().map(|html| html.to_string()).collect();
    assert_eq!(compare(&cases, 296), 0);
    assert_eq!(compare(&cases, 400), 0);
}

#[test]
#[ignore]
fn random_documents_match_the_original() {
    let mut random = Random(5);
    let cases: Vec<String> = (0..120).map(|_| wrap(&inline(&mut random, 0))).collect();
    assert_eq!(compare(&cases, 296), 0);
}

#[test]
#[ignore]
fn single_case_from_file() {
    let Ok(path) = std::env::var("CASE") else { return };
    let html_text = std::fs::read_to_string(path).unwrap();
    let font = real_font();
    let requests = vec![json!({"html": html_text, "width": 296, "height": 4000, "widths": widths(&font)})];
    let Some(results) = oracle(&requests) else { return };
    let (expected, actual) = (original_tree(&results[0]), native_tree(&html_text, 296, &font));
    eprintln!("ORIGINAL:\n{}\nPORTE:\n{}", expected.join("\n"), actual.join("\n"));
    let (pe, pa) = (original_paint(&results[0]), native_paint(&html_text, 296, &font));
    eprintln!("PINTURA ORIGINAL:\n{}\nPINTURA PORTE:\n{}", pe.join("\n"), pa.join("\n"));
}
