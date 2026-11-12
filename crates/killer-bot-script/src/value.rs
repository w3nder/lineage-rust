use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Num(i64),
    Text(String),
    Bool(bool),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Num(_) => "número",
            Value::Text(_) => "texto",
            Value::Bool(_) => "booleano",
        }
    }

    pub fn truthy(&self) -> Option<bool> {
        match self {
            Value::Bool(value) => Some(*value),
            Value::Num(value) => Some(*value != 0),
            Value::Text(_) => None,
        }
    }

    pub fn num(&self) -> Option<i64> {
        match self {
            Value::Num(value) => Some(*value),
            Value::Bool(value) => Some(i64::from(*value)),
            Value::Text(_) => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Num(value) => write!(f, "{value}"),
            Value::Text(value) => write!(f, "{value}"),
            Value::Bool(value) => write!(f, "{}", if *value { "true" } else { "false" }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truthiness_and_numbers() {
        assert_eq!(Value::Bool(true).truthy(), Some(true));
        assert_eq!(Value::Num(0).truthy(), Some(false));
        assert_eq!(Value::Num(3).truthy(), Some(true));
        assert_eq!(Value::Text("x".into()).truthy(), None);
        assert_eq!(Value::Bool(true).num(), Some(1));
        assert_eq!(Value::Text("1".into()).num(), None);
        assert_eq!(Value::Text("a".into()).type_name(), "texto");
        assert_eq!(Value::Num(7).to_string(), "7");
        assert_eq!(Value::Bool(false).to_string(), "false");
    }
}
