use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl ScriptError {
    pub fn new(line: usize, column: usize, message: impl Into<String>) -> Self {
        Self { line, column, message: message.into() }
    }
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.column == 0 {
            write!(f, "linha {}: {}", self.line, self.message)
        } else {
            write!(f, "linha {}, coluna {}: {}", self.line, self.column, self.message)
        }
    }
}

impl std::error::Error for ScriptError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Number(i64),
    Text(String),
    Ident(String),
    Op(&'static str),
    Newline,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

const OPERATORS: [&str; 15] = ["==", "!=", "<=", ">=", "<", ">", "=", "+", "-", "*", "/", "%", "(", ")", ","];

pub fn lex(source: &str) -> Result<Vec<Token>, ScriptError> {
    let chars: Vec<char> = source.chars().collect();
    let mut tokens = Vec::new();
    let (mut i, mut line, mut line_start) = (0usize, 1usize, 0usize);
    while i < chars.len() {
        let c = chars[i];
        let column = i - line_start + 1;
        match c {
            '\n' => {
                tokens.push(Token { kind: TokenKind::Newline, line, column });
                i += 1;
                line += 1;
                line_start = i;
            }
            ' ' | '\t' | '\r' => i += 1,
            '#' => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '"' => {
                let mut text = String::new();
                i += 1;
                loop {
                    match chars.get(i) {
                        None | Some('\n') => return Err(ScriptError::new(line, column, "texto sem aspas de fechamento")),
                        Some('"') => {
                            i += 1;
                            break;
                        }
                        Some(&other) => {
                            text.push(other);
                            i += 1;
                        }
                    }
                }
                tokens.push(Token { kind: TokenKind::Text(text), line, column });
            }
            c if c.is_ascii_digit() => {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let text: String = chars[start..i].iter().collect();
                let value = text.parse::<i64>().map_err(|_| ScriptError::new(line, column, "número grande demais"))?;
                tokens.push(Token { kind: TokenKind::Number(value), line, column });
            }
            c if c.is_alphabetic() || c == '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                tokens.push(Token { kind: TokenKind::Ident(chars[start..i].iter().collect::<String>().to_lowercase()), line, column });
            }
            _ => {
                let rest: String = chars[i..chars.len().min(i + 2)].iter().collect();
                match OPERATORS.iter().find(|op| rest.starts_with(**op)) {
                    Some(op) => {
                        tokens.push(Token { kind: TokenKind::Op(op), line, column });
                        i += op.len();
                    }
                    None => return Err(ScriptError::new(line, column, format!("caractere inesperado {c:?}"))),
                }
            }
        }
    }
    tokens.push(Token { kind: TokenKind::Newline, line, column: i - line_start + 1 });
    tokens.push(Token { kind: TokenKind::End, line, column: i - line_start + 1 });
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<TokenKind> {
        lex(source).unwrap().into_iter().map(|token| token.kind).collect()
    }

    #[test]
    fn lexes_numbers_text_idents_and_operators() {
        assert_eq!(
            kinds("set x = 10 + hp >= \"oi\""),
            vec![
                TokenKind::Ident("set".into()),
                TokenKind::Ident("x".into()),
                TokenKind::Op("="),
                TokenKind::Number(10),
                TokenKind::Op("+"),
                TokenKind::Ident("hp".into()),
                TokenKind::Op(">="),
                TokenKind::Text("oi".into()),
                TokenKind::Newline,
                TokenKind::End,
            ]
        );
    }

    #[test]
    fn identifiers_are_case_insensitive_but_text_is_kept() {
        assert_eq!(kinds("HP \"Maiúscula\"")[..2], [TokenKind::Ident("hp".into()), TokenKind::Text("Maiúscula".into())]);
    }

    #[test]
    fn comments_and_blank_lines_are_skipped() {
        let tokens = kinds("# começo\n\nwait 5 # depois\n");
        assert_eq!(tokens.iter().filter(|kind| **kind == TokenKind::Newline).count(), 4);
        assert!(tokens.contains(&TokenKind::Number(5)));
        assert!(!tokens.iter().any(|kind| matches!(kind, TokenKind::Ident(name) if name == "depois")));
    }

    #[test]
    fn unterminated_text_reports_line_and_column() {
        let error = lex("wait 1\nsay \"oi").unwrap_err();
        assert_eq!((error.line, error.column), (2, 5));
        assert_eq!(error.to_string(), "linha 2, coluna 5: texto sem aspas de fechamento");
    }

    #[test]
    fn unknown_character_is_an_error_not_a_panic() {
        for source in ["@", "set x = $", "\u{0}", "é = 1 ¬", "999999999999999999999999"] {
            let _ = lex(source);
        }
        assert!(lex("set x = @").is_err());
        assert!(lex("99999999999999999999").is_err());
    }
}
