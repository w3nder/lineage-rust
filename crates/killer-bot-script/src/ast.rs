#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    Death,
    Revive,
    EnterCombat,
    LeaveCombat,
    TargetDead,
    TargetSelected,
    TargetLost,
    Chat,
    PlayerAttack,
    Arrived,
}

impl EventKind {
    pub fn from_name(name: &str) -> Option<EventKind> {
        Some(match name {
            "death" => EventKind::Death,
            "revive" => EventKind::Revive,
            "enter_combat" => EventKind::EnterCombat,
            "leave_combat" => EventKind::LeaveCombat,
            "target_dead" => EventKind::TargetDead,
            "target_selected" => EventKind::TargetSelected,
            "target_lost" => EventKind::TargetLost,
            "chat" => EventKind::Chat,
            "player_attack" => EventKind::PlayerAttack,
            "arrived" => EventKind::Arrived,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(i64),
    Text(String),
    Bool(bool),
    Name { name: String, args: Vec<Expr>, line: usize },
    Unary(UnaryOp, Box<Expr>, usize),
    Binary(BinaryOp, Box<Expr>, Box<Expr>, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Set { name: String, value: Expr, line: usize },
    If { branches: Vec<(Expr, Vec<Stmt>)>, otherwise: Vec<Stmt> },
    While { condition: Expr, body: Vec<Stmt> },
    Repeat { count: Expr, body: Vec<Stmt>, line: usize },
    Wait { millis: Expr, line: usize },
    Stop,
    Command { name: String, args: Vec<Expr>, line: usize },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Handler {
    pub event: EventKind,
    pub filter: Option<String>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Program {
    pub body: Vec<Stmt>,
    pub handlers: Vec<Handler>,
}
