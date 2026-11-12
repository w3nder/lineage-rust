use crate::ast::{BinaryOp, EventKind, Expr, Handler, Program, Stmt, UnaryOp};
use crate::lexer::{ScriptError, Token, TokenKind, lex};

pub const QUERY_ARITY: [(&str, usize); 13] = [
    ("item_count", 1),
    ("skill_level", 1),
    ("nearby_count", 1),
    ("has_buff", 1),
    ("skill_ready", 1),
    ("is_player", 1),
    ("is_monster", 1),
    ("in_range", 1),
    ("distance_to", 3),
    ("has_item", 1),
    ("buff_time", 1),
    ("npc_count", 1),
    ("has_skill", 1),
];

const KEYWORDS: [&str; 12] = ["set", "if", "elif", "else", "then", "end", "while", "repeat", "wait", "stop", "on", "true"];
const MAX_DEPTH: usize = 64;

pub fn query_arity(name: &str) -> usize {
    QUERY_ARITY.iter().find(|(key, _)| *key == name).map_or(0, |(_, arity)| *arity)
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
    depth: usize,
}

pub fn parse(source: &str) -> Result<Program, ScriptError> {
    let mut parser = Parser { tokens: lex(source)?, at: 0, depth: 0 };
    parser.program()
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.at.min(self.tokens.len() - 1)]
    }

    fn next(&mut self) -> Token {
        let token = self.peek().clone();
        if self.at < self.tokens.len() - 1 {
            self.at += 1;
        }
        token
    }

    fn error<T>(&self, token: &Token, message: impl Into<String>) -> Result<T, ScriptError> {
        Err(ScriptError::new(token.line, token.column, message))
    }

    fn is_ident(&self, word: &str) -> bool {
        matches!(&self.peek().kind, TokenKind::Ident(name) if name == word)
    }

    fn is_op(&self, op: &str) -> bool {
        matches!(&self.peek().kind, TokenKind::Op(found) if *found == op)
    }

    fn skip_newlines(&mut self) {
        while self.peek().kind == TokenKind::Newline {
            self.next();
        }
    }

    fn end_of_statement(&mut self) -> Result<(), ScriptError> {
        match self.peek().kind {
            TokenKind::Newline => {
                self.next();
                Ok(())
            }
            TokenKind::End => Ok(()),
            _ => {
                let token = self.peek().clone();
                self.error(&token, "esperava o fim da linha")
            }
        }
    }

    fn program(&mut self) -> Result<Program, ScriptError> {
        let mut program = Program::default();
        loop {
            self.skip_newlines();
            if self.peek().kind == TokenKind::End {
                return Ok(program);
            }
            if self.is_ident("on") {
                program.handlers.push(self.handler()?);
            } else {
                program.body.push(self.statement()?);
            }
        }
    }

    fn handler(&mut self) -> Result<Handler, ScriptError> {
        let on = self.next();
        let name = match self.next() {
            Token { kind: TokenKind::Ident(name), .. } => name,
            token => return self.error(&token, "esperava o nome do evento depois de on"),
        };
        let Some(event) = EventKind::from_name(&name) else {
            return self.error(&on, format!("evento desconhecido: {name}"));
        };
        let filter = match self.peek().kind.clone() {
            TokenKind::Text(text) => {
                self.next();
                Some(text)
            }
            _ => None,
        };
        self.end_of_statement()?;
        let body = self.block(&["end"], &on)?;
        self.next();
        Ok(Handler { event, filter, body })
    }

    fn block(&mut self, stops: &[&str], opened: &Token) -> Result<Vec<Stmt>, ScriptError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            let token = self.peek().clone();
            return self.error(&token, "blocos aninhados demais");
        }
        let mut body = Vec::new();
        loop {
            self.skip_newlines();
            match &self.peek().kind {
                TokenKind::End => return self.error(opened, "bloco aberto aqui e sem end"),
                TokenKind::Ident(name) if stops.contains(&name.as_str()) => break,
                _ => body.push(self.statement()?),
            }
        }
        self.depth -= 1;
        Ok(body)
    }

    fn statement(&mut self) -> Result<Stmt, ScriptError> {
        let token = self.next();
        let TokenKind::Ident(word) = token.kind.clone() else {
            return self.error(&token, "esperava um comando");
        };
        let line = token.line;
        match word.as_str() {
            "set" => {
                let name = match self.next() {
                    Token { kind: TokenKind::Ident(name), .. } if !KEYWORDS.contains(&name.as_str()) => name,
                    other => return self.error(&other, "esperava o nome da variável depois de set"),
                };
                if !self.is_op("=") {
                    let found = self.peek().clone();
                    return self.error(&found, "esperava = na atribuição");
                }
                self.next();
                let value = self.expression()?;
                self.end_of_statement()?;
                Ok(Stmt::Set { name, value, line })
            }
            "if" => {
                let mut branches = Vec::new();
                let condition = self.expression()?;
                if self.is_ident("then") {
                    self.next();
                }
                self.end_of_statement()?;
                branches.push((condition, self.block(&["elif", "else", "end"], &token)?));
                let mut otherwise = Vec::new();
                loop {
                    if self.is_ident("elif") {
                        self.next();
                        let condition = self.expression()?;
                        if self.is_ident("then") {
                            self.next();
                        }
                        self.end_of_statement()?;
                        branches.push((condition, self.block(&["elif", "else", "end"], &token)?));
                    } else if self.is_ident("else") {
                        self.next();
                        self.end_of_statement()?;
                        otherwise = self.block(&["end"], &token)?;
                    } else {
                        break;
                    }
                }
                self.next();
                self.end_of_statement()?;
                Ok(Stmt::If { branches, otherwise })
            }
            "while" => {
                let condition = self.expression()?;
                self.end_of_statement()?;
                let body = self.block(&["end"], &token)?;
                self.next();
                self.end_of_statement()?;
                Ok(Stmt::While { condition, body })
            }
            "repeat" => {
                let count = self.expression()?;
                self.end_of_statement()?;
                let body = self.block(&["end"], &token)?;
                self.next();
                self.end_of_statement()?;
                Ok(Stmt::Repeat { count, body, line })
            }
            "wait" => {
                let millis = self.expression()?;
                self.end_of_statement()?;
                Ok(Stmt::Wait { millis, line })
            }
            "stop" => {
                self.end_of_statement()?;
                Ok(Stmt::Stop)
            }
            "end" | "elif" | "else" | "then" => self.error(&token, format!("{word} sem o bloco correspondente")),
            "on" => self.error(&token, "on só vale no começo do script, fora de blocos"),
            _ => {
                let mut args = Vec::new();
                while !matches!(self.peek().kind, TokenKind::Newline | TokenKind::End) {
                    if self.is_op(",") {
                        self.next();
                        continue;
                    }
                    args.push(self.unary()?);
                }
                self.end_of_statement()?;
                Ok(Stmt::Command { name: word, args, line })
            }
        }
    }

    fn expression(&mut self) -> Result<Expr, ScriptError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            let token = self.peek().clone();
            return self.error(&token, "expressão aninhada demais");
        }
        let mut left = self.and()?;
        while self.is_ident("or") {
            let op = self.next();
            let right = self.and()?;
            left = Expr::Binary(BinaryOp::Or, Box::new(left), Box::new(right), op.line);
        }
        self.depth -= 1;
        Ok(left)
    }

    fn and(&mut self) -> Result<Expr, ScriptError> {
        let mut left = self.not()?;
        while self.is_ident("and") {
            let op = self.next();
            let right = self.not()?;
            left = Expr::Binary(BinaryOp::And, Box::new(left), Box::new(right), op.line);
        }
        Ok(left)
    }

    fn not(&mut self) -> Result<Expr, ScriptError> {
        if self.is_ident("not") {
            let op = self.next();
            let inner = self.not()?;
            return Ok(Expr::Unary(UnaryOp::Not, Box::new(inner), op.line));
        }
        self.comparison()
    }

    fn comparison(&mut self) -> Result<Expr, ScriptError> {
        let left = self.sum()?;
        let op = match &self.peek().kind {
            TokenKind::Op("==") => BinaryOp::Eq,
            TokenKind::Op("!=") => BinaryOp::Ne,
            TokenKind::Op("<") => BinaryOp::Lt,
            TokenKind::Op("<=") => BinaryOp::Le,
            TokenKind::Op(">") => BinaryOp::Gt,
            TokenKind::Op(">=") => BinaryOp::Ge,
            _ => return Ok(left),
        };
        let token = self.next();
        let right = self.sum()?;
        Ok(Expr::Binary(op, Box::new(left), Box::new(right), token.line))
    }

    fn sum(&mut self) -> Result<Expr, ScriptError> {
        let mut left = self.term()?;
        loop {
            let op = match &self.peek().kind {
                TokenKind::Op("+") => BinaryOp::Add,
                TokenKind::Op("-") => BinaryOp::Sub,
                _ => return Ok(left),
            };
            let token = self.next();
            let right = self.term()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right), token.line);
        }
    }

    fn term(&mut self) -> Result<Expr, ScriptError> {
        let mut left = self.unary()?;
        loop {
            let op = match &self.peek().kind {
                TokenKind::Op("*") => BinaryOp::Mul,
                TokenKind::Op("/") => BinaryOp::Div,
                TokenKind::Op("%") => BinaryOp::Mod,
                _ => return Ok(left),
            };
            let token = self.next();
            let right = self.unary()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right), token.line);
        }
    }

    fn unary(&mut self) -> Result<Expr, ScriptError> {
        if self.is_op("-") {
            let token = self.next();
            let inner = self.unary()?;
            return Ok(Expr::Unary(UnaryOp::Neg, Box::new(inner), token.line));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ScriptError> {
        let token = self.next();
        match token.kind.clone() {
            TokenKind::Number(value) => Ok(Expr::Num(value)),
            TokenKind::Text(text) => Ok(Expr::Text(text)),
            TokenKind::Op("(") => {
                let inner = self.expression()?;
                if !self.is_op(")") {
                    let found = self.peek().clone();
                    return self.error(&found, "esperava )");
                }
                self.next();
                Ok(inner)
            }
            TokenKind::Ident(name) if name == "true" => Ok(Expr::Bool(true)),
            TokenKind::Ident(name) if name == "false" => Ok(Expr::Bool(false)),
            TokenKind::Ident(name) if KEYWORDS.contains(&name.as_str()) => self.error(&token, format!("{name} não é um valor")),
            TokenKind::Ident(name) => {
                let mut args = Vec::new();
                for _ in 0..query_arity(&name) {
                    if matches!(self.peek().kind, TokenKind::Newline | TokenKind::End) {
                        return self.error(&token, format!("{name} precisa de {} argumento(s)", query_arity(&name)));
                    }
                    args.push(self.unary()?);
                }
                Ok(Expr::Name { name, args, line: token.line })
            }
            _ => self.error(&token, "esperava um valor"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stmts(source: &str) -> Vec<Stmt> {
        parse(source).unwrap().body
    }

    #[test]
    fn parses_set_if_elif_else_while_repeat() {
        let body = stmts("set n = 3\nif hp < 50 then\nsit\nelif hp < 80\nstand\nelse\nwait 10\nend\nwhile true\nstop\nend\nrepeat n\nattack\nend\n");
        assert_eq!(body.len(), 4);
        assert!(matches!(&body[0], Stmt::Set { name, .. } if name == "n"));
        match &body[1] {
            Stmt::If { branches, otherwise } => {
                assert_eq!(branches.len(), 2);
                assert_eq!(otherwise.len(), 1);
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(&body[2], Stmt::While { .. }));
        assert!(matches!(&body[3], Stmt::Repeat { .. }));
    }

    #[test]
    fn precedence_and_parentheses() {
        let body = stmts("set x = 1 + 2 * 3\nset y = (1 + 2) * 3\nset z = not a and b or c\n");
        let Stmt::Set { value, .. } = &body[0] else { panic!() };
        assert_eq!(
            *value,
            Expr::Binary(BinaryOp::Add, Box::new(Expr::Num(1)), Box::new(Expr::Binary(BinaryOp::Mul, Box::new(Expr::Num(2)), Box::new(Expr::Num(3)), 1)), 1)
        );
        let Stmt::Set { value, .. } = &body[1] else { panic!() };
        assert!(matches!(value, Expr::Binary(BinaryOp::Mul, left, _, _) if matches!(**left, Expr::Binary(BinaryOp::Add, ..))));
        let Stmt::Set { value, .. } = &body[2] else { panic!() };
        assert!(matches!(value, Expr::Binary(BinaryOp::Or, left, _, _) if matches!(**left, Expr::Binary(BinaryOp::And, ..))));
    }

    #[test]
    fn command_arguments_are_expressions() {
        let body = stmts("move_to 1 -2 (x + 3)\nsay \"all\", \"oi\"\nuse_item item_count 57\n");
        let Stmt::Command { name, args, .. } = &body[0] else { panic!() };
        assert_eq!(name, "move_to");
        assert_eq!(args.len(), 3);
        assert!(matches!(args[1], Expr::Unary(UnaryOp::Neg, ..)));
        let Stmt::Command { args, .. } = &body[1] else { panic!() };
        assert_eq!(args.len(), 2);
        let Stmt::Command { args, .. } = &body[2] else { panic!() };
        assert!(matches!(&args[0], Expr::Name { name, args, .. } if name == "item_count" && args.len() == 1));
    }

    #[test]
    fn on_event_with_filter() {
        let program = parse("on death\nsay \"all\" \"morri\"\nend\non chat \"gm\"\nstop\nend\nattack\n").unwrap();
        assert_eq!(program.handlers.len(), 2);
        assert_eq!(program.handlers[0].event, EventKind::Death);
        assert_eq!(program.handlers[1].filter.as_deref(), Some("gm"));
        assert_eq!(program.body.len(), 1);
        assert!(parse("on voar\nend\n").is_err());
    }

    #[test]
    fn missing_end_reports_the_opening_line() {
        let error = parse("wait 1\nif hp < 3\nsit\n").unwrap_err();
        assert_eq!(error.line, 2);
        assert!(error.message.contains("sem end"));
        assert!(parse("end\n").is_err());
        assert!(parse("set = 3\n").is_err());
        assert!(parse("set x 3\n").is_err());
        assert!(parse("item_count\n").is_ok());
        assert!(parse("set a = item_count\n").is_err());
    }

    #[test]
    fn garbage_never_panics() {
        for source in ["(((((", ")", "if if if", "set x = (1 +", "on", "on death", "while", "repeat", "wait", "\"", "-", "set x = not", "if x\nelse\nelse\nend", "# só comentário", ""] {
            let _ = parse(source);
        }
        let deep = format!("set x = {}1{}\n", "(".repeat(500), ")".repeat(500));
        assert!(parse(&deep).is_err());
        let nested = format!("{}stop\n{}", "while true\n".repeat(200), "end\n".repeat(200));
        assert!(parse(&nested).is_err());
    }
}
