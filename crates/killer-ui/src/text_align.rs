use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::true_font::LineFont;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitmapLine {
    pub text: String,
    pub spaces: Vec<i32>,
}

#[derive(Clone)]
enum Token {
    Word(String, i32),
    Space,
    Newline,
}

fn tokens<F: LineFont + ?Sized>(text: &str, font: &F) -> Vec<Token> {
    let mut result = Vec::new();
    let mut word = String::new();
    for c in text.chars() {
        if matches!(c, ' ' | '\n') {
            if !word.is_empty() {
                let width = font.measure_text(&word);
                result.push(Token::Word(std::mem::take(&mut word), width));
            }
            result.push(if c == ' ' { Token::Space } else { Token::Newline });
        } else {
            word.push(c);
        }
    }
    if !word.is_empty() {
        let width = font.measure_text(&word);
        result.push(Token::Word(word, width));
    }
    result
}

fn trim_edges(line: &mut BitmapLine, trim: bool, paragraph_start: bool, both: bool) -> i32 {
    if !trim || line.spaces.is_empty() {
        return 0;
    }
    let mut removed = 0;
    if !paragraph_start && line.text.starts_with(' ') {
        line.text.remove(0);
        removed += line.spaces.remove(0);
    }
    if both && !line.spaces.is_empty() && line.text.ends_with(' ') {
        line.text.pop();
        removed += line.spaces.pop().unwrap_or(0);
    }
    removed
}

fn distribute(line: &mut BitmapLine, amount: i32, shrink: bool) {
    let count = line.spaces.len() as i32;
    if count == 0 {
        return;
    }
    let amount = amount.max(0);
    let share = amount / count;
    let remainder = amount % count;
    for (index, space) in line.spaces.iter_mut().enumerate() {
        let adjustment = share + i32::from((index as i32) < remainder);
        *space += if shrink { -adjustment } else { adjustment };
    }
}

pub fn parser_lines<F: LineFont + ?Sized>(text: &str, width: i32, font: &F, language: i32, trim: bool) -> Vec<BitmapLine> {
    parser_lines_from(text, width, font, language, trim, 0)
}

pub fn parser_lines_from<F: LineFont + ?Sized>(text: &str, width: i32, font: &F, language: i32, trim: bool, start: i32) -> Vec<BitmapLine> {
    let text = text.split('\0').next().unwrap_or("");
    if text.is_empty() {
        return Vec::new();
    }
    if language != 1 || width == 0 {
        return vec![BitmapLine { text: text.to_owned(), spaces: Vec::new() }];
    }
    let space = font.char_width(' ').unwrap_or(0);
    let minimum = space / 2 + 2;
    let mut tokens = tokens(text, font);
    let mut index = 0;
    let mut paragraph_start = true;
    let mut result = Vec::new();
    let mut first = true;
    while index < tokens.len() {
        let mut line = BitmapLine { text: String::new(), spaces: Vec::new() };
        let mut used = if std::mem::take(&mut first) { start } else { 0 };
        loop {
            let Some(token) = tokens.get(index).cloned() else {
                if !line.text.is_empty() {
                    trim_edges(&mut line, trim, paragraph_start, false);
                    result.push(line);
                }
                return result;
            };
            match token {
                Token::Space => {
                    used += space;
                    index += 1;
                    if used >= width {
                        let removed = trim_edges(&mut line, trim, paragraph_start, true);
                        distribute(&mut line, width - used + space + removed, false);
                        paragraph_start = false;
                        result.push(line);
                        break;
                    }
                    line.text.push(' ');
                    line.spaces.push(space);
                }
                Token::Newline => {
                    index += 1;
                    trim_edges(&mut line, trim, paragraph_start, false);
                    paragraph_start = true;
                    result.push(line);
                    break;
                }
                Token::Word(word, token_width) => {
                    let previous = used;
                    used += token_width;
                    if used <= width {
                        line.text.push_str(&word);
                        index += 1;
                        continue;
                    }
                    if line.text.is_empty() {
                        let mut measured = 0;
                        let mut split = 0;
                        for (offset, c) in word.char_indices() {
                            measured += font.char_width(c).unwrap_or(0);
                            if measured >= width {
                                break;
                            }
                            split = offset + c.len_utf8();
                        }
                        if split == 0 {
                            return result;
                        }
                        if split == word.len() {
                            line.text = word;
                            result.push(line);
                            return result;
                        }
                        line.text = word[..split].to_owned();
                        tokens[index] = Token::Word(word[split..].to_owned(), token_width);
                        result.push(line);
                        break;
                    }
                    if line.spaces.is_empty() {
                        line.text.push_str(&word);
                        index += 1;
                        paragraph_start = false;
                        result.push(line);
                        break;
                    }
                    let excess = used - width;
                    if (space - minimum) * (line.spaces.len() as i32) < excess {
                        let removed = trim_edges(&mut line, trim, paragraph_start, true);
                        distribute(&mut line, width - previous + removed, false);
                    } else {
                        line.text.push_str(&word);
                        index += 1;
                        let removed = trim_edges(&mut line, trim, paragraph_start, true);
                        distribute(&mut line, excess - removed, true);
                    }
                    paragraph_start = false;
                    result.push(line);
                    break;
                }
            }
        }
    }
    result
}

pub fn bitmap_lines<F: LineFont + ?Sized>(text: &str, width: i32, font: &F, language: i32, trim: bool) -> Vec<BitmapLine> {
    bitmap_lines_from(text, width, font, language, trim, 0)
}

pub fn bitmap_lines_from<F: LineFont + ?Sized>(text: &str, width: i32, font: &F, language: i32, trim: bool, start: i32) -> Vec<BitmapLine> {
    glyph_lines_from(text, width, font, language, trim, start, false)
}

pub fn true_font_lines<F: LineFont + ?Sized>(text: &str, width: i32, font: &F, language: i32) -> Vec<BitmapLine> {
    glyph_lines_from(text, width, font, language, false, 0, true)
}

fn glyph_lines_from<F: LineFont + ?Sized>(
    text: &str,
    width: i32,
    font: &F,
    language: i32,
    trim: bool,
    start: i32,
    punctuation_stays: bool,
) -> Vec<BitmapLine> {
    let mut result = Vec::new();
    let space = font.char_width(' ').unwrap_or(0);
    let mut first = true;
    for parsed in parser_lines_from(text, width, font, language, trim, start) {
        let mut spaces = parsed.spaces.into_iter();
        let mut line = BitmapLine { text: String::new(), spaces: Vec::new() };
        let mut used = if std::mem::take(&mut first) { start } else { 0 };
        for c in parsed.text.chars() {
            if c == '\n' {
                result.push(line);
                line = BitmapLine { text: String::new(), spaces: Vec::new() };
                used = 0;
            } else if c == ' ' {
                let advance = spaces.next().unwrap_or(space);
                line.text.push(c);
                line.spaces.push(advance);
                used += advance;
            } else if c > '\u{1f}' {
                let Some(advance) = font.char_width(c) else {
                    continue;
                };
                if width != 0 && used + advance > width && !(punctuation_stays && matches!(c, '.' | ',')) {
                    result.push(line);
                    line = BitmapLine { text: String::new(), spaces: Vec::new() };
                    used = 0;
                }
                line.text.push(c);
                used += advance;
            }
        }
        result.push(line);
    }
    result
}

pub fn measure_height(text: &str, width: i32, font: &BitmapFont, language: i32, trim: bool, leading: bool) -> i32 {
    let h = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
    let step = h + if leading { 6 } else { 0 };
    let space = font.glyph(' ').map_or(0, |glyph| glyph.width as i32);
    let mut height = h;
    for (index, parsed) in parser_lines(text, width, font, language, trim).into_iter().enumerate() {
        if index > 0 && !parsed.text.is_empty() {
            height += step;
        }
        let mut used = 0;
        let mut spaces = parsed.spaces.into_iter();
        for c in parsed.text.chars() {
            if c == '\n' {
                height += step;
                used = 0;
            } else if c == ' ' {
                used += spaces.next().unwrap_or(space);
            } else if c > '\u{1f}'
                && let Some(glyph) = font.glyph(c)
            {
                if width != 0 && used + glyph.width as i32 > width {
                    height += step;
                    used = 0;
                }
                used += glyph.width as i32;
            }
        }
    }
    height
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Extent {
    pub width: i32,
    pub height: i32,
    pub end_x: i32,
}

pub fn extent(text: &str, width: i32, font: &BitmapFont, language: i32, start: i32, leading: bool) -> Extent {
    let h = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
    let step = h + if leading { 6 } else { 0 };
    let space = font.glyph(' ').map_or(0, |glyph| glyph.width as i32);
    let (mut height, mut widest, mut x) = (h, 0, start);
    for (index, parsed) in parser_lines_from(text, width, font, language, false, start).into_iter().enumerate() {
        if index > 0 && !parsed.text.is_empty() {
            height += step;
            x = 0;
        }
        let mut spaces = parsed.spaces.into_iter();
        for c in parsed.text.chars() {
            if c == '\n' {
                height += step;
                x = 0;
            } else if c == ' ' {
                x += spaces.next().unwrap_or(space);
                widest = widest.max(x);
            } else if c > '\u{1f}'
                && let Some(glyph) = font.glyph(c)
            {
                if width != 0 && x + glyph.width as i32 > width {
                    height += step;
                    x = 0;
                }
                x += glyph.width as i32;
                widest = widest.max(x);
            }
        }
    }
    Extent { width: widest, height, end_x: x }
}

pub fn glyph_positions(text: &str, width: i32, font: &BitmapFont, language: i32, start: i32, leading: bool) -> Vec<(i32, i32, char)> {
    glyph_positions_trimmed(text, width, font, language, start, leading, false)
}

pub fn glyph_positions_trimmed(text: &str, width: i32, font: &BitmapFont, language: i32, start: i32, leading: bool, trim: bool) -> Vec<(i32, i32, char)> {
    let h = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
    let step = h + if leading { 6 } else { 0 };
    let space = font.glyph(' ').map_or(0, |glyph| glyph.width as i32);
    let mut out = Vec::new();
    for (index, line) in bitmap_lines_from(text, width, font, language, trim, start).into_iter().enumerate() {
        let mut x = if index == 0 { start } else { 0 };
        let mut spaces = line.spaces.iter();
        for c in line.text.chars() {
            if c == ' ' {
                x += spaces.next().copied().unwrap_or(space);
            } else if let Some(glyph) = font.glyph(c) {
                out.push((x, index as i32 * step, c));
                x += glyph.width as i32;
            }
        }
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub x: i32,
    pub y: i32,
    pub advance: i32,
    pub c: char,
}

pub fn glyph_cells(text: &str, width: i32, font: &BitmapFont, language: i32, start: i32, leading: bool, trim: bool) -> Vec<Cell> {
    let h = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
    let step = h + if leading { 6 } else { 0 };
    let space = font.glyph(' ').map_or(0, |glyph| glyph.width as i32);
    let mut out = Vec::new();
    for (index, line) in bitmap_lines_from(text, width, font, language, trim, start).into_iter().enumerate() {
        let mut x = if index == 0 { start } else { 0 };
        let mut spaces = line.spaces.iter();
        for c in line.text.chars() {
            let advance = if c == ' ' {
                spaces.next().copied().unwrap_or(space)
            } else if let Some(glyph) = font.glyph(c) {
                glyph.width as i32
            } else {
                continue;
            };
            out.push(Cell { x, y: index as i32 * step, advance, c });
            x += advance;
        }
    }
    out
}

fn line_width<F: LineFont + ?Sized>(line: &BitmapLine, font: &F) -> i32 {
    let mut spaces = line.spaces.iter();
    line.text
        .chars()
        .map(|c| if c == ' ' { spaces.next().copied().unwrap_or_else(|| font.char_width(' ').unwrap_or(0)) } else { font.char_width(c).unwrap_or(0) })
        .sum()
}

pub fn true_font_width<F: LineFont + ?Sized>(text: &str, width: i32, font: &F) -> i32 {
    true_font_lines(text, width, font, 1).iter().map(|line| line_width(line, font)).max().unwrap_or(0)
}

pub fn draw_true_font_wrapped<F: LineFont + ?Sized>(list: &mut DrawList, x: i32, y: i32, color: Argb, text: &str, width: i32, font: &F) {
    let lines = true_font_lines(text, width, font, 1);
    paint_lines(list, (x, y), &lines, color, font, font.height());
}

pub fn paint_lines<F: LineFont + ?Sized>(
    list: &mut DrawList,
    origin: (i32, i32),
    lines: &[BitmapLine],
    color: Argb,
    font: &F,
    step: i32,
) {
    let normal_space = font.char_width(' ').unwrap_or(0);
    for (index, line) in lines.iter().enumerate() {
        let y = origin.1 + index as i32 * step;
        if line.spaces.iter().all(|space| *space == normal_space) {
            font.draw(list, origin.0, y, color, &line.text);
            continue;
        }
        let mut x = origin.0;
        let mut spaces = line.spaces.iter();
        for word in line.text.split(' ') {
            if !word.is_empty() {
                font.draw(list, x, y, color, word);
                x += font.measure_text(word);
            }
            if let Some(space) = spaces.next() {
                x += space;
            }
        }
    }
}

pub fn height(lines: &[BitmapLine], font: &BitmapFont, leading: bool) -> i32 {
    let h = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
    h + (lines.len() as i32 - 1).max(0) * (h + if leading { 6 } else { 0 })
}

pub fn paint(
    list: &mut DrawList,
    origin: (i32, i32),
    lines: &[BitmapLine],
    color: Argb,
    font: &BitmapFont,
    leading: bool,
) {
    let h = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
    let step = h + if leading { 6 } else { 0 };
    for (index, line) in lines.iter().enumerate() {
        let normal_space = font.glyph(' ').map_or(0, |glyph| glyph.width as i32);
        if line.spaces.iter().all(|space| *space == normal_space) {
            list.draw_text(origin.0, origin.1 + index as i32 * step, color, &line.text, font);
            continue;
        }
        let mut x = origin.0;
        let mut spaces = line.spaces.iter();
        for word in line.text.split(' ') {
            if !word.is_empty() {
                list.draw_text(x, origin.1 + index as i32 * step, color, word, font);
                x += font.measure(word) as i32;
            }
            if let Some(space) = spaces.next() {
                x += space;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::Glyphs;

    include!("../tests/fixtures/ssq-text-align.rs");

    fn font() -> BitmapFont {
        BitmapFont::new(
            "L2Font-e.SmallFont-e".into(),
            Glyphs::parse(include_bytes!("../tests/fixtures/smallfont-e.gly")).unwrap(),
        )
    }

    #[test]
    fn every_ascii_glyph_and_parser_branch_matches_c4_engine_fixture() {
        let font = font();
        assert_eq!(CASES.len(), 1980);
        for &(text, width, trim, expected) in CASES {
            let actual = parser_lines(text, width, &font, 1, trim);
            let expected: Vec<_> = expected
                .iter()
                .map(|(text, spaces)| BitmapLine { text: (*text).into(), spaces: spaces.to_vec() })
                .collect();
            assert_eq!(actual, expected, "text={text:?}, width={width}, trim={trim}");
        }
    }

    #[test]
    fn measured_height_matches_native_canvas_for_all_fixture_cases() {
        let font = font();
        assert_eq!(HEIGHT_CASES.len(), 990);
        for &(text, width, expected) in HEIGHT_CASES {
            assert_eq!(measure_height(text, width, &font, 1, false, true), expected, "text={text:?}, width={width}");
        }
    }

    #[test]
    fn true_font_text_wraps_by_words_then_by_glyph_and_keeps_punctuation_on_the_line() {
        let font = font();
        let lines = true_font_lines("AA BB CC", 30, &font, 1);
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|line| line_width(line, &font) <= 30 + font.char_width(' ').unwrap_or(0)));
        assert_eq!(true_font_lines("AA BB CC", 0, &font, 1).len(), 1);
        let w = font.char_width('A').unwrap();
        let plain = true_font_lines(&"A".repeat(5), 3 * w, &font, 1);
        assert_eq!(plain.iter().map(|line| line.text.len()).collect::<Vec<_>>(), vec![2, 2, 1]);
        assert_eq!(true_font_width("AAAAA", 3 * w, &font), 2 * w);
    }

    #[test]
    fn non_english_and_unbounded_text_bypass_word_parser() {
        let font = font();
        for (language, width) in [(0, 20), (2, 20), (1, 0)] {
            assert_eq!(
                parser_lines("A B\nC", width, &font, language, false),
                [BitmapLine { text: "A B\nC".into(), spaces: Vec::new() }]
            );
        }
    }

    #[test]
    fn spaces_use_native_justification_and_multiline_leading() {
        let font = font();
        let lines = bitmap_lines("One two three four five", 55, &font, 1, false);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].spaces, [9, 8]);
        assert_eq!(measure_height("One two three four five", 55, &font, 1, false, true), 51);
        assert_eq!(measure_height("\n\nA\n", 55, &font, 1, false, true), 32);
        let mut list = DrawList { capture_text: true, ..Default::default() };
        paint(&mut list, (0, 0), &lines, crate::color::TEXT, &font, true);
        let two = list.text_runs.iter().find(|run| run.text == "two").unwrap();
        assert_eq!(two.rect[0], font.measure("One") as f32 + 9.0);
        let last = list.text_runs.iter().find(|run| run.text == "five").unwrap();
        assert_eq!(last.rect[1], 38.0);
    }
}
