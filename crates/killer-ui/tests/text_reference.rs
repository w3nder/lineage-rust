use killer_ui::font::{BitmapFont, GlyphRect, Glyphs};
use killer_ui::text_align;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

struct Random(u64);

impl Random {
    fn next(&mut self, below: u64) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) % below
    }
}

fn fixed_font() -> BitmapFont {
    let glyphs: HashMap<char, GlyphRect> = (0x20u32..0x7f)
        .filter_map(char::from_u32)
        .map(|c| (c, GlyphRect { x: 0, width: 7, y: 0, height: 13 }))
        .collect();
    BitmapFont::new("fixed".into(), Glyphs { tex_width: 512, tex_height: 512, glyphs })
}

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

fn sample(random: &mut Random) -> String {
    const WORDS: [&str; 12] = ["a", "to", "the", "Gatekeeper", "Roxxy:", "I'll", "teleport", "you", "wherever", "supercalifragilistic", "W", "mmmm"];
    let count = 1 + random.next(14);
    let mut text = String::new();
    for index in 0..count {
        if index > 0 {
            text.push(' ');
            if random.next(9) == 0 {
                text.push(' ');
            }
        }
        text.push_str(WORDS[random.next(WORDS.len() as u64) as usize]);
    }
    text
}

#[test]
#[ignore]
fn text_extent_and_glyph_positions_match_the_engine() {
    let mut random = Random(11);
    let fonts = [("fixa", fixed_font()), ("real", real_font())];
    let mut cases = Vec::new();
    let mut inputs = Vec::new();
    for index in 0..300 {
        let (name, font) = &fonts[index % 2];
        let text = sample(&mut random);
        let width = 28 + random.next(260) as i32;
        let start = if random.next(3) == 0 { 0 } else { random.next(width as u64 / 2 + 1) as i32 };
        cases.push(json!({"text": text, "width": width, "x": start, "widths": if *name == "real" { widths(font) } else { json!({}) }}));
        inputs.push((index % 2, text, width, start));
    }
    let Some(results) = oracle(&cases) else { return };
    let mut failures = 0;
    for ((font_index, text, width, start), result) in inputs.into_iter().zip(results) {
        let font = &fonts[font_index].1;
        let extent = text_align::extent(&text, width, font, 1, start, true);
        let native = text_align::glyph_positions_trimmed(&text, width, font, 1, start, true, true);
        let measure = &result["measure"];
        let expected_extent = (measure["w"].as_i64().unwrap() as i32, measure["h"].as_i64().unwrap() as i32, measure["end"].as_i64().unwrap() as i32);
        let letters: Vec<char> = text.chars().filter(|c| *c != ' ').collect();
        let expected: Vec<(i32, i32, char)> = result["glyphs"]
            .as_array()
            .unwrap()
            .iter()
            .zip(&letters)
            .map(|(g, c)| (g[0].as_f64().unwrap() as i32, g[1].as_f64().unwrap() as i32, *c))
            .collect();
        if (extent.width, extent.height, extent.end_x) != expected_extent || native != expected {
            failures += 1;
            let first = native.iter().zip(&expected).position(|(a, b)| a != b);
            eprintln!("{} width={width} start={start} {text:?}\n   extent porte {:?} original {:?}; primeiro glifo diferente {:?}: porte {:?} original {:?}", fonts[font_index].0, (extent.width, extent.height, extent.end_x), expected_extent, first, first.map(|i| native[i]), first.map(|i| expected[i]));
        }
    }
    assert_eq!(failures, 0);
}
