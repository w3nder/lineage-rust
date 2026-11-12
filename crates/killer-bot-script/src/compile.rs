use crate::ast::{BinaryOp, EventKind, Expr, Program, Stmt, UnaryOp};
use crate::value::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Push(Value),
    Load { name: String, line: usize },
    Store(String),
    Unary(UnaryOp, usize),
    Binary(BinaryOp, usize),
    Query { name: String, argc: usize, line: usize },
    Act { name: String, argc: usize, line: usize },
    Jump(usize),
    JumpIfFalse(usize, usize),
    Wait(usize),
    Stop,
    Return,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledHandler {
    pub event: EventKind,
    pub filter: Option<String>,
    pub code: Vec<Op>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Compiled {
    pub main: Vec<Op>,
    pub handlers: Vec<CompiledHandler>,
}

struct Emitter {
    code: Vec<Op>,
    counters: usize,
}

pub fn compile(program: &Program) -> Compiled {
    let mut emitter = Emitter { code: Vec::new(), counters: 0 };
    emitter.block(&program.body);
    emitter.code.push(Op::Return);
    let main = std::mem::take(&mut emitter.code);
    let mut handlers = Vec::new();
    for handler in &program.handlers {
        emitter.block(&handler.body);
        emitter.code.push(Op::Return);
        handlers.push(CompiledHandler { event: handler.event, filter: handler.filter.clone(), code: std::mem::take(&mut emitter.code) });
    }
    Compiled { main, handlers }
}

impl Emitter {
    fn block(&mut self, body: &[Stmt]) {
        for stmt in body {
            self.stmt(stmt);
        }
    }

    fn here(&self) -> usize {
        self.code.len()
    }

    fn stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Set { name, value, .. } => {
                self.expr(value);
                self.code.push(Op::Store(name.clone()));
            }
            Stmt::If { branches, otherwise } => {
                let mut exits = Vec::new();
                for (condition, body) in branches {
                    self.expr(condition);
                    let test = self.here();
                    self.code.push(Op::JumpIfFalse(0, condition_line(condition)));
                    self.block(body);
                    exits.push(self.here());
                    self.code.push(Op::Jump(0));
                    let next = self.here();
                    if let Op::JumpIfFalse(target, _) = &mut self.code[test] {
                        *target = next;
                    }
                }
                self.block(otherwise);
                let end = self.here();
                for exit in exits {
                    self.code[exit] = Op::Jump(end);
                }
            }
            Stmt::While { condition, body } => {
                let top = self.here();
                self.expr(condition);
                let test = self.here();
                self.code.push(Op::JumpIfFalse(0, condition_line(condition)));
                self.block(body);
                self.code.push(Op::Jump(top));
                let end = self.here();
                self.code[test] = Op::JumpIfFalse(end, condition_line(condition));
            }
            Stmt::Repeat { count, body, line } => {
                let hidden = format!("#repeat{}", self.counters);
                self.counters += 1;
                self.expr(count);
                self.code.push(Op::Store(hidden.clone()));
                let top = self.here();
                self.code.push(Op::Load { name: hidden.clone(), line: *line });
                self.code.push(Op::Push(Value::Num(0)));
                self.code.push(Op::Binary(BinaryOp::Gt, *line));
                let test = self.here();
                self.code.push(Op::JumpIfFalse(0, *line));
                self.block(body);
                self.code.push(Op::Load { name: hidden.clone(), line: *line });
                self.code.push(Op::Push(Value::Num(1)));
                self.code.push(Op::Binary(BinaryOp::Sub, *line));
                self.code.push(Op::Store(hidden));
                self.code.push(Op::Jump(top));
                let end = self.here();
                self.code[test] = Op::JumpIfFalse(end, *line);
            }
            Stmt::Wait { millis, line } => {
                self.expr(millis);
                self.code.push(Op::Wait(*line));
            }
            Stmt::Stop => self.code.push(Op::Stop),
            Stmt::Command { name, args, line } => {
                for arg in args {
                    self.expr(arg);
                }
                self.code.push(Op::Act { name: name.clone(), argc: args.len(), line: *line });
            }
        }
    }

    fn expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Num(value) => self.code.push(Op::Push(Value::Num(*value))),
            Expr::Text(value) => self.code.push(Op::Push(Value::Text(value.clone()))),
            Expr::Bool(value) => self.code.push(Op::Push(Value::Bool(*value))),
            Expr::Name { name, args, line } => {
                if args.is_empty() {
                    self.code.push(Op::Load { name: name.clone(), line: *line });
                } else {
                    for arg in args {
                        self.expr(arg);
                    }
                    self.code.push(Op::Query { name: name.clone(), argc: args.len(), line: *line });
                }
            }
            Expr::Unary(op, inner, line) => {
                self.expr(inner);
                self.code.push(Op::Unary(*op, *line));
            }
            Expr::Binary(op, left, right, line) => {
                self.expr(left);
                self.expr(right);
                self.code.push(Op::Binary(*op, *line));
            }
        }
    }
}

fn condition_line(expr: &Expr) -> usize {
    match expr {
        Expr::Name { line, .. } | Expr::Unary(_, _, line) | Expr::Binary(_, _, _, line) => *line,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    fn main_of(source: &str) -> Vec<Op> {
        compile(&parse(source).unwrap()).main
    }

    #[test]
    fn if_else_compiles_to_forward_jumps() {
        let code = main_of("if hp < 50\nsit\nelse\nstand\nend\n");
        let Op::JumpIfFalse(target, _) = code[3] else { panic!("{code:?}") };
        assert!(target > 3 && target < code.len());
        assert!(matches!(code[target - 1], Op::Jump(end) if end >= target));
        assert_eq!(code.last(), Some(&Op::Return));
    }

    #[test]
    fn while_loops_back() {
        let code = main_of("while true\nattack\nend\n");
        assert!(code.iter().any(|op| matches!(op, Op::Jump(0))));
    }

    #[test]
    fn repeat_counts_with_a_hidden_variable() {
        let code = main_of("repeat 3\nattack\nend\nrepeat 2\nsit\nend\n");
        let hidden: Vec<&String> = code.iter().filter_map(|op| if let Op::Store(name) = op { Some(name) } else { None }).collect();
        assert!(hidden.iter().all(|name| name.starts_with('#')));
        assert_ne!(hidden[0], hidden[hidden.len() - 1]);
    }

    #[test]
    fn handler_bodies_end_with_return() {
        let compiled = compile(&parse("on death\nsit\nend\non revive\nstand\nend\nstop\n").unwrap());
        assert_eq!(compiled.handlers.len(), 2);
        assert!(compiled.handlers.iter().all(|handler| handler.code.last() == Some(&Op::Return)));
        assert_eq!(compiled.main, vec![Op::Stop, Op::Return]);
    }
}
