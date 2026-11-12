use crate::ast::{BinaryOp, EventKind, UnaryOp};
use crate::compile::{Compiled, Op};
use crate::lexer::ScriptError;
use crate::value::Value;
use std::collections::{HashMap, VecDeque};
use std::time::Duration;

pub const OPS_PER_STEP: usize = 2000;
pub const QUEUE_PER_HANDLER: usize = 8;

pub trait Host {
    fn query(&mut self, name: &str, args: &[Value]) -> Result<Value, String>;
    fn act(&mut self, name: &str, args: &[Value]) -> Result<(), String>;
    fn print(&mut self, text: &str);
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Running,
    Waiting,
    Finished,
    Failed(ScriptError),
}

#[derive(Debug)]
struct Task {
    handler: Option<usize>,
    pc: usize,
    stack: Vec<Value>,
    wake: Option<Duration>,
    locals: HashMap<String, Value>,
    line: usize,
}

impl Task {
    fn new(handler: Option<usize>, payload: Option<Value>) -> Task {
        let mut locals = HashMap::new();
        if let Some(payload) = payload {
            locals.insert("event_text".to_owned(), payload);
        }
        Task { handler, pc: 0, stack: Vec::new(), wake: None, locals, line: 0 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskInfo {
    pub handler: bool,
    pub line: usize,
    pub wake: Option<Duration>,
}

pub struct Vm {
    compiled: Compiled,
    globals: HashMap<String, Value>,
    tasks: Vec<Task>,
    queues: Vec<VecDeque<Value>>,
    dropped: usize,
    state: Status,
    paused: bool,
}

enum Flow {
    Continue,
    Yield,
    Done,
    StopAll,
}

impl Vm {
    pub fn new(compiled: Compiled) -> Vm {
        let queues = vec![VecDeque::new(); compiled.handlers.len()];
        Vm { compiled, globals: HashMap::new(), tasks: vec![Task::new(None, None)], queues, dropped: 0, state: Status::Running, paused: false }
    }

    pub fn tasks(&self) -> Vec<TaskInfo> {
        self.tasks.iter().map(|task| TaskInfo { handler: task.handler.is_some(), line: task.line, wake: task.wake }).collect()
    }

    pub fn dropped_events(&self) -> usize {
        self.dropped
    }

    pub fn variables(&self) -> Vec<(String, Value)> {
        let mut list: Vec<(String, Value)> = self.globals.iter().filter(|(name, _)| !name.starts_with('#')).map(|(name, value)| (name.clone(), value.clone())).collect();
        list.sort_by(|a, b| a.0.cmp(&b.0));
        list
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn status(&self) -> &Status {
        &self.state
    }

    pub fn variable(&self, name: &str) -> Option<&Value> {
        self.globals.get(name)
    }

    pub fn fire(&mut self, event: EventKind, payload: Value) {
        if matches!(self.state, Status::Failed(_) | Status::Finished) {
            return;
        }
        let text = payload.to_string().to_lowercase();
        for index in 0..self.compiled.handlers.len() {
            let handler = &self.compiled.handlers[index];
            if handler.event != event || handler.filter.as_ref().is_some_and(|filter| !text.contains(&filter.to_lowercase())) {
                continue;
            }
            if self.tasks.iter().any(|task| task.handler == Some(index)) {
                if self.queues[index].len() >= QUEUE_PER_HANDLER {
                    self.dropped += 1;
                } else {
                    self.queues[index].push_back(payload.clone());
                }
            } else {
                self.tasks.push(Task::new(Some(index), Some(payload.clone())));
            }
        }
    }

    pub fn step(&mut self, host: &mut dyn Host, now: Duration) -> Status {
        if let Status::Failed(_) | Status::Finished = self.state {
            return self.state.clone();
        }
        if self.paused {
            return Status::Waiting;
        }
        let mut budget = OPS_PER_STEP;
        let mut exhausted = false;
        let mut index = 0;
        while index < self.tasks.len() {
            if self.tasks[index].wake.is_some_and(|wake| now < wake) {
                index += 1;
                continue;
            }
            self.tasks[index].wake = None;
            match self.run(index, host, now, &mut budget) {
                Ok(Flow::Done) => {
                    let task = self.tasks.remove(index);
                    if let Some(handler) = task.handler
                        && let Some(payload) = self.queues[handler].pop_front()
                    {
                        self.tasks.push(Task::new(Some(handler), Some(payload)));
                    }
                    continue;
                }
                Ok(Flow::StopAll) => {
                    self.tasks.clear();
                    self.queues.iter_mut().for_each(VecDeque::clear);
                    self.state = Status::Finished;
                    return Status::Finished;
                }
                Ok(Flow::Yield) => {
                    if budget == 0 {
                        exhausted = true;
                    }
                }
                Ok(Flow::Continue) => {}
                Err(error) => {
                    self.tasks.clear();
                    self.state = Status::Failed(error.clone());
                    return Status::Failed(error);
                }
            }
            if exhausted {
                break;
            }
            index += 1;
        }
        let main_alive = self.tasks.iter().any(|task| task.handler.is_none());
        self.state = if exhausted || self.tasks.iter().any(|task| task.wake.is_none()) {
            Status::Running
        } else if !main_alive && self.compiled.handlers.is_empty() {
            Status::Finished
        } else {
            Status::Waiting
        };
        self.state.clone()
    }

    fn lookup(&self, index: usize, name: &str) -> Option<Value> {
        self.tasks[index].locals.get(name).or_else(|| self.globals.get(name)).cloned()
    }

    fn run(&mut self, index: usize, host: &mut dyn Host, now: Duration, budget: &mut usize) -> Result<Flow, ScriptError> {
        loop {
            if *budget == 0 {
                return Ok(Flow::Yield);
            }
            *budget -= 1;
            let handler = self.tasks[index].handler;
            let pc = self.tasks[index].pc;
            let code = match handler {
                None => &self.compiled.main,
                Some(handler) => &self.compiled.handlers[handler].code,
            };
            let Some(op) = code.get(pc).cloned() else { return Ok(Flow::Done) };
            self.tasks[index].pc += 1;
            match self.exec(index, op, host, now)? {
                Flow::Continue => {}
                other => return Ok(other),
            }
        }
    }

    fn exec(&mut self, index: usize, op: Op, host: &mut dyn Host, now: Duration) -> Result<Flow, ScriptError> {
        let mut line = self.tasks[index].line;
        let fail = |line: usize, message: String| ScriptError::new(line, 0, message);
        let result = (|| -> Result<Flow, ScriptError> {
            match op {
                Op::Push(value) => self.tasks[index].stack.push(value),
                Op::Load { name, line: at } => {
                    line = at;
                    let value = match self.lookup(index, &name) {
                        Some(value) => value,
                        None => host.query(&name, &[]).map_err(|message| fail(at, message))?,
                    };
                    self.tasks[index].stack.push(value);
                }
                Op::Store(name) => {
                    let value = self.pop(index, line)?;
                    self.globals.insert(name, value);
                }
                Op::Unary(op, at) => {
                    line = at;
                    let value = self.pop(index, line)?;
                    let result = match op {
                        UnaryOp::Not => Value::Bool(!truth(&value, at)?),
                        UnaryOp::Neg => Value::Num(num(&value, at)?.checked_neg().ok_or_else(|| fail(at, "estouro de número".into()))?),
                    };
                    self.tasks[index].stack.push(result);
                }
                Op::Binary(op, at) => {
                    line = at;
                    let right = self.pop(index, line)?;
                    let left = self.pop(index, line)?;
                    let result = binary(op, left, right, at)?;
                    self.tasks[index].stack.push(result);
                }
                Op::Query { name, argc, line: at } => {
                    line = at;
                    let args = self.pop_args(index, argc, at)?;
                    let value = host.query(&name, &args).map_err(|message| fail(at, message))?;
                    self.tasks[index].stack.push(value);
                }
                Op::Act { name, argc, line: at } => {
                    line = at;
                    let args = self.pop_args(index, argc, at)?;
                    if name == "print" {
                        host.print(&args.iter().map(Value::to_string).collect::<Vec<_>>().join(" "));
                    } else {
                        host.act(&name, &args).map_err(|message| fail(at, message))?;
                    }
                }
                Op::Jump(target) => self.tasks[index].pc = target,
                Op::JumpIfFalse(target, at) => {
                    if at != 0 {
                        line = at;
                    }
                    let value = self.pop(index, line)?;
                    if !truth(&value, line)? {
                        self.tasks[index].pc = target;
                    }
                }
                Op::Wait(at) => {
                    line = at;
                    let millis = self.pop(index, line)?;
                    let millis = num(&millis, at)?.max(0) as u64;
                    self.tasks[index].wake = Some(now + Duration::from_millis(millis));
                    return Ok(Flow::Yield);
                }
                Op::Stop => return Ok(Flow::StopAll),
                Op::Return => return Ok(Flow::Done),
            }
            Ok(Flow::Continue)
        })();
        self.tasks[index].line = line;
        result
    }

    fn pop(&mut self, index: usize, line: usize) -> Result<Value, ScriptError> {
        self.tasks[index].stack.pop().ok_or_else(|| ScriptError::new(line, 0, "pilha vazia (script inválido)"))
    }

    fn pop_args(&mut self, index: usize, argc: usize, line: usize) -> Result<Vec<Value>, ScriptError> {
        let stack = &mut self.tasks[index].stack;
        if stack.len() < argc {
            return Err(ScriptError::new(line, 0, "pilha vazia (script inválido)"));
        }
        let at = stack.len() - argc;
        Ok(stack.split_off(at))
    }
}

fn truth(value: &Value, line: usize) -> Result<bool, ScriptError> {
    value.truthy().ok_or_else(|| ScriptError::new(line, 0, format!("a condição precisa ser booleano ou número, veio {}", value.type_name())))
}

fn num(value: &Value, line: usize) -> Result<i64, ScriptError> {
    value.num().ok_or_else(|| ScriptError::new(line, 0, format!("esperava número, veio {}", value.type_name())))
}

fn binary(op: BinaryOp, left: Value, right: Value, line: usize) -> Result<Value, ScriptError> {
    let fail = |message: String| ScriptError::new(line, 0, message);
    match op {
        BinaryOp::Add => {
            if matches!(left, Value::Text(_)) || matches!(right, Value::Text(_)) {
                return Ok(Value::Text(format!("{left}{right}")));
            }
            Ok(Value::Num(num(&left, line)?.checked_add(num(&right, line)?).ok_or_else(|| fail("estouro de número".into()))?))
        }
        BinaryOp::Sub => Ok(Value::Num(num(&left, line)?.checked_sub(num(&right, line)?).ok_or_else(|| fail("estouro de número".into()))?)),
        BinaryOp::Mul => Ok(Value::Num(num(&left, line)?.checked_mul(num(&right, line)?).ok_or_else(|| fail("estouro de número".into()))?)),
        BinaryOp::Div | BinaryOp::Mod => {
            let (a, b) = (num(&left, line)?, num(&right, line)?);
            if b == 0 {
                return Err(fail("divisão por zero".into()));
            }
            let result = if op == BinaryOp::Div { a.checked_div(b) } else { a.checked_rem(b) };
            Ok(Value::Num(result.ok_or_else(|| fail("estouro de número".into()))?))
        }
        BinaryOp::Eq | BinaryOp::Ne => {
            let equal = match (&left, &right) {
                (Value::Text(a), Value::Text(b)) => a.to_lowercase() == b.to_lowercase(),
                (Value::Text(_), other) | (other, Value::Text(_)) => return Err(fail(format!("não dá para comparar texto com {}", other.type_name()))),
                _ => num(&left, line)? == num(&right, line)?,
            };
            Ok(Value::Bool(equal == (op == BinaryOp::Eq)))
        }
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            let order = match (&left, &right) {
                (Value::Text(a), Value::Text(b)) => a.to_lowercase().cmp(&b.to_lowercase()),
                _ => num(&left, line)?.cmp(&num(&right, line)?),
            };
            Ok(Value::Bool(match op {
                BinaryOp::Lt => order.is_lt(),
                BinaryOp::Le => order.is_le(),
                BinaryOp::Gt => order.is_gt(),
                _ => order.is_ge(),
            }))
        }
        BinaryOp::And => Ok(Value::Bool(truth(&left, line)? && truth(&right, line)?)),
        BinaryOp::Or => Ok(Value::Bool(truth(&left, line)? || truth(&right, line)?)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build;

    #[derive(Default)]
    struct Test {
        acts: Vec<(String, Vec<Value>)>,
        printed: Vec<String>,
        hp: i64,
    }

    impl Host for Test {
        fn query(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
            match name {
                "hp" => Ok(Value::Num(self.hp)),
                "item_count" => Ok(Value::Num(args.first().and_then(Value::num).unwrap_or(0) * 2)),
                _ => Err(format!("desconhecido: {name}")),
            }
        }

        fn act(&mut self, name: &str, args: &[Value]) -> Result<(), String> {
            if name == "boom" {
                return Err("recusado".to_owned());
            }
            self.acts.push((name.to_owned(), args.to_vec()));
            Ok(())
        }

        fn print(&mut self, text: &str) {
            self.printed.push(text.to_owned());
        }
    }

    fn run(source: &str) -> (Vm, Test, Status) {
        let mut vm = Vm::new(build(source).unwrap());
        let mut host = Test { hp: 40, ..Default::default() };
        let status = vm.step(&mut host, Duration::ZERO);
        (vm, host, status)
    }

    #[test]
    fn arithmetic_comparison_and_strings() {
        let (vm, host, status) = run("set a = 2 + 3 * 4\nset b = a > 10 and hp < 50\nset c = \"x\" + a\nprint a b c\nset d = item_count 5 + 1\nsay \"all\" \"Oi\"\n");
        assert_eq!(status, Status::Finished);
        assert_eq!(vm.variable("a"), Some(&Value::Num(14)));
        assert_eq!(host.printed, vec!["14 true x14"]);
        assert_eq!(vm.variable("d"), Some(&Value::Num(11)));
        assert_eq!(host.acts, vec![("say".to_owned(), vec![Value::Text("all".into()), Value::Text("Oi".into())])]);
    }

    #[test]
    fn wait_yields_until_the_time_passes() {
        let mut vm = Vm::new(build("attack\nwait 1500\nsit\n").unwrap());
        let mut host = Test::default();
        assert_eq!(vm.step(&mut host, Duration::ZERO), Status::Waiting);
        assert_eq!(host.acts.len(), 1);
        assert_eq!(vm.step(&mut host, Duration::from_millis(1499)), Status::Waiting);
        assert_eq!(host.acts.len(), 1);
        assert_eq!(vm.step(&mut host, Duration::from_millis(1500)), Status::Finished);
        assert_eq!(host.acts.len(), 2);
    }

    #[test]
    fn infinite_loop_without_wait_is_cut_by_the_budget_and_resumes_next_step() {
        let mut host = Test::default();
        let mut vm = Vm::new(build("set n = 0\nwhile true\nset n = n + 1\nend\n").unwrap());
        assert_eq!(vm.step(&mut host, Duration::ZERO), Status::Running);
        let first = vm.variable("n").cloned().unwrap();
        assert_eq!(vm.step(&mut host, Duration::ZERO), Status::Running);
        let Value::Num(a) = first else { panic!() };
        let Some(Value::Num(b)) = vm.variable("n").cloned() else { panic!() };
        assert!(a > 100 && b > a);
    }

    #[test]
    fn division_by_zero_fails_with_the_line() {
        let (_, _, status) = run("set a = 1\nset b = 5 / (a - 1)\n");
        let Status::Failed(error) = status else { panic!("{status:?}") };
        assert_eq!(error.line, 2);
        assert!(error.message.contains("zero"));
    }

    #[test]
    fn unknown_variable_fails_with_the_line() {
        let mut vm = Vm::new(build("wait 0\nset a = nada + 1\n").unwrap());
        let mut host = Test::default();
        assert_eq!(vm.step(&mut host, Duration::ZERO), Status::Waiting);
        let status = vm.step(&mut host, Duration::ZERO);
        let Status::Failed(error) = status else { panic!("{status:?}") };
        assert_eq!(error.line, 2);
        assert!(error.message.contains("nada"));
    }

    #[test]
    fn non_boolean_condition_is_a_type_error() {
        let (_, _, status) = run("if \"texto\"\nsit\nend\n");
        assert!(matches!(status, Status::Failed(error) if error.message.contains("texto")));
        let (_, _, status) = run("set a = \"a\" < 3\n");
        assert!(matches!(status, Status::Failed(_)));
    }

    #[test]
    fn act_errors_stop_the_script_with_the_line() {
        let (_, host, status) = run("sit\nboom 1\nstand\n");
        let Status::Failed(error) = status else { panic!() };
        assert_eq!((error.line, error.message.as_str()), (2, "recusado"));
        assert_eq!(host.acts.len(), 1);
    }

    #[test]
    fn if_elif_else_while_and_repeat_run() {
        let (vm, host, _) = run("set total = 0\nrepeat 4\nset total = total + 2\nend\nif total > 100\nsay \"a\"\nelif total == 8\nsay \"b\"\nelse\nsay \"c\"\nend\nset i = 0\nwhile i < 3\nset i = i + 1\nend\n");
        assert_eq!(vm.variable("total"), Some(&Value::Num(8)));
        assert_eq!(vm.variable("i"), Some(&Value::Num(3)));
        assert_eq!(host.acts.len(), 1);
        assert_eq!(host.acts[0].1, vec![Value::Text("b".into())]);
    }

    #[test]
    fn pausing_freezes_the_script_and_variables_hide_the_internal_counters() {
        let mut vm = Vm::new(build("set b = 2\nset a = 1\nrepeat 2\nsit\nend\nwait 100000\n").unwrap());
        let mut host = Test::default();
        vm.set_paused(true);
        assert_eq!(vm.step(&mut host, Duration::ZERO), Status::Waiting);
        assert!(host.acts.is_empty() && vm.variables().is_empty());
        vm.set_paused(false);
        vm.step(&mut host, Duration::ZERO);
        assert_eq!(host.acts.len(), 2);
        assert_eq!(vm.variables(), vec![("a".to_owned(), Value::Num(1)), ("b".to_owned(), Value::Num(2))]);
        assert!(!vm.paused());
    }

    #[test]
    fn stop_ends_the_whole_script() {
        let (_, host, status) = run("sit\nstop\nstand\n");
        assert_eq!(status, Status::Finished);
        assert_eq!(host.acts.len(), 1);
    }

    #[test]
    fn death_handler_runs_without_stopping_the_main_task() {
        let mut vm = Vm::new(build("on death\nsay \"all\" \"morri\"\nend\nwhile true\nwait 1000\nend\n").unwrap());
        let mut host = Test::default();
        assert_eq!(vm.step(&mut host, Duration::ZERO), Status::Waiting);
        vm.fire(EventKind::Death, Value::Bool(true));
        vm.step(&mut host, Duration::ZERO);
        assert_eq!(host.acts.len(), 1);
        vm.step(&mut host, Duration::from_millis(1000));
        assert!(matches!(vm.status(), Status::Waiting));
    }

    #[test]
    fn chat_filter_matches_only_when_the_text_contains_the_filter() {
        let mut vm = Vm::new(build("on chat \"gm\"\nprint event_text\nend\nwait 100000\n").unwrap());
        let mut host = Test::default();
        vm.step(&mut host, Duration::ZERO);
        vm.fire(EventKind::Chat, Value::Text("oi pessoal".into()));
        vm.fire(EventKind::Chat, Value::Text("Chegou o GM aqui".into()));
        vm.step(&mut host, Duration::ZERO);
        assert_eq!(host.printed, vec!["Chegou o GM aqui"]);
    }

    #[test]
    fn flooding_an_event_does_not_grow_the_queue_without_limit() {
        let mut vm = Vm::new(build("on chat\nwait 5000\nend\nwait 100000\n").unwrap());
        let mut host = Test::default();
        vm.step(&mut host, Duration::ZERO);
        for n in 0..100 {
            vm.fire(EventKind::Chat, Value::Num(n));
        }
        assert_eq!(vm.dropped_events(), 100 - 1 - QUEUE_PER_HANDLER);
        vm.step(&mut host, Duration::ZERO);
        assert!(vm.tasks.len() <= 2);
    }

    #[test]
    fn handler_can_wait() {
        let mut vm = Vm::new(build("on revive\nwait 300\nsay \"all\" \"voltei\"\nend\nwait 100000\n").unwrap());
        let mut host = Test::default();
        vm.step(&mut host, Duration::ZERO);
        vm.fire(EventKind::Revive, Value::Bool(true));
        vm.step(&mut host, Duration::ZERO);
        assert!(host.acts.is_empty());
        vm.step(&mut host, Duration::from_millis(300));
        assert_eq!(host.acts.len(), 1);
    }

    #[test]
    fn events_after_a_failure_or_finish_are_ignored() {
        let (mut vm, mut host, _) = run("on death\nsit\nend\nstop\n");
        vm.fire(EventKind::Death, Value::Bool(true));
        assert_eq!(vm.step(&mut host, Duration::ZERO), Status::Finished);
        assert!(host.acts.is_empty());
    }

    #[test]
    fn tasks_report_the_line_and_the_wake_time_of_a_waiting_script() {
        struct Quiet;
        impl Host for Quiet {
            fn query(&mut self, _: &str, _: &[Value]) -> Result<Value, String> {
                Ok(Value::Bool(true))
            }
            fn act(&mut self, _: &str, _: &[Value]) -> Result<(), String> {
                Ok(())
            }
            fn print(&mut self, _: &str) {}
        }
        let compiled = crate::build("while true\n  print \"a\"\n  wait 500\nend\n").unwrap();
        let mut vm = Vm::new(compiled);
        let status = vm.step(&mut Quiet, Duration::from_millis(100));
        assert_eq!(status, Status::Waiting);
        let tasks = vm.tasks();
        assert_eq!(tasks.len(), 1);
        assert_eq!((tasks[0].line, tasks[0].handler), (3, false));
        assert_eq!(tasks[0].wake, Some(Duration::from_millis(600)));
    }
}
