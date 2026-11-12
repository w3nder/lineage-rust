pub mod ast;

pub mod compile;
pub mod lexer;
pub mod parser;
pub mod value;
pub mod vm;


pub use ast::EventKind;
pub use compile::{Compiled, compile};

pub use lexer::ScriptError;
pub use parser::parse;
pub use value::Value;
pub use vm::{Host, OPS_PER_STEP, Status, TaskInfo, Vm};



pub fn build(source: &str) -> Result<Compiled, ScriptError> {
    Ok(compile(&parse(source)?))
}
