#[derive(Debug)]
pub enum Token {
    Int(u64),
    Double(f64),
    ClassName(String),
    ClassInstance(usize),
    String(String),
    Null,
    Array(usize),
}

impl Token {
    pub fn get_type_as_str(&self) -> &str {
        use Token::*;
        match self {
            Int(_) => "int",
            Double(_) => "double",
            ClassName(_) => "class_name",
            ClassInstance(_) => "class_instance",
            String(_) => "string",
            Null => "null",
            Array(_) => "array",
        }
    }
}

impl ToString for Token {
    fn to_string(&self) -> String {
        use Token::*;
        match self {
            Int(v) => v.to_string(),
            Double(v) => v.to_string(),
            ClassName(v) => v.to_string(),
            ClassInstance(v) => v.to_string(),
            String(v) => v.to_string(),
            Null => "null".to_string(),
            Array(v) => v.to_string(),
        }
    }
}

impl From<Token> for u64 {
    fn from(value: Token) -> Self {
        match value {
            Token::Int(v) => v,
            v => panic!("Cannot convert {v:?} to u64"),
        }
    }
}

impl From<Token> for f64 {
    fn from(value: Token) -> Self {
        match value {
            Token::Double(v) => v,
            other => panic!("Cannot convert {other:?} to f64"),
        }
    }
}

impl From<Token> for String {
    fn from(value: Token) -> Self {
        match value {
            Token::String(v) => v,
            other => panic!("Cannot convert {other:?} to String"),
        }
    }
}

impl From<Token> for bool {
    fn from(value: Token) -> Self {
        match value {
            Token::Int(v) => v != 0,
            v => panic!("Cannot convert {v:?} to bool"),
        }
    }
}

impl From<Token> for usize {
    fn from(value: Token) -> Self {
        match value {
            Token::Array(v) | Token::ClassInstance(v) => v,
            v => panic!("Cannot convert {v:?} to usize"),
        }
    }
}

impl From<Token> for i32 {
    fn from(value: Token) -> Self {
        match value {
            Token::Int(v) => v as i32,
            v => panic!("Cannot convert {v:?} to i32"),
        }
    }
}

impl From<Token> for i8 {
    fn from(value: Token) -> Self {
        match value {
            Token::Int(v) => v as i8,
            v => panic!("Cannot convert {v:?} to i8"),
        }
    }
}

impl From<Token> for Option<u64> {
    fn from(value: Token) -> Self {
        match value {
            Token::Int(v) => Some(v),
            _ => None,
        }
    }
}

impl From<Token> for Option<f64> {
    fn from(value: Token) -> Self {
        match value {
            Token::Double(v) => Some(v),
            _ => None,
        }
    }
}

impl From<Token> for Option<String> {
    fn from(value: Token) -> Self {
        match value {
            Token::String(v) => Some(v),
            _ => None,
        }
    }
}

impl From<Token> for Option<bool> {
    fn from(value: Token) -> Self {
        match value {
            Token::Int(v) => Some(v != 0),
            _ => None,
        }
    }
}

impl From<Token> for Option<usize> {
    fn from(value: Token) -> Self {
        match value {
            Token::Array(v) => Some(v),
            _ => None,
        }
    }
}

impl From<Token> for Option<i32> {
    fn from(value: Token) -> Self {
        match value {
            Token::Int(v) => Some(v as i32),
            _ => None,
        }
    }
}

impl From<Token> for Option<i8> {
    fn from(value: Token) -> Self {
        match value {
            Token::Int(v) => Some(v as i8),
            _ => None,
        }
    }
}
