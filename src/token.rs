use serde_json::Value;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Token {
    Int(u64),
    Double(f64),
    ClassName(String),
    ClassInstance(usize),
    String(String),
    Null,
    Array(usize),
    Json(String),
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
            Json(_) => "json",
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
            Json(v) => v.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct ConversionError {
    from: &'static str,
    to: &'static str,
    value: String,
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cannot convert from {} to {}: value was {}",
            self.from, self.to, self.value
        )
    }
}

impl Error for ConversionError {}

impl TryFrom<Token> for u64 {
    type Error = ConversionError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Int(v) => Ok(v),
            other => Err(ConversionError {
                from: "Token",
                to: "u64",
                value: format!("{:?}", other),
            }),
        }
    }
}

impl TryFrom<Token> for f64 {
    type Error = ConversionError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Double(v) => Ok(v),
            other => Err(ConversionError {
                from: "Token",
                to: "f64",
                value: format!("{:?}", other),
            }),
        }
    }
}

impl TryFrom<Token> for String {
    type Error = ConversionError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::String(v) => Ok(v),
            other => Err(ConversionError {
                from: "Token",
                to: "String",
                value: format!("{:?}", other),
            }),
        }
    }
}

impl TryFrom<Token> for bool {
    type Error = ConversionError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Int(v) => Ok(v != 0),
            other => Err(ConversionError {
                from: "Token",
                to: "bool",
                value: format!("{:?}", other),
            }),
        }
    }
}

impl TryFrom<Token> for usize {
    type Error = ConversionError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Array(v) | Token::ClassInstance(v) => Ok(v),
            other => Err(ConversionError {
                from: "Token",
                to: "usize",
                value: format!("{:?}", other),
            }),
        }
    }
}

impl TryFrom<Token> for i32 {
    type Error = ConversionError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Int(v) => Ok(v as i32),
            other => Err(ConversionError {
                from: "Token",
                to: "i32",
                value: format!("{:?}", other),
            }),
        }
    }
}

impl TryFrom<Token> for i8 {
    type Error = ConversionError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Int(v) => Ok(v as i8),
            other => Err(ConversionError {
                from: "Token",
                to: "i8",
                value: format!("{:?}", other),
            }),
        }
    }
}

impl TryFrom<Token> for Value {
    type Error = ConversionError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Json(ref s) => serde_json::from_str(s).map_err(|err| ConversionError {
                from: "Token",
                to: "Value",
                value: format!("{:?}", value.clone()),
            }),
            other => Err(ConversionError {
                from: "Token",
                to: "Value",
                value: format!("{:?}", other),
            }),
        }
    }
}

impl TryFrom<Token> for Option<u64> {
    type Error = ConversionError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Int(v) => Ok(Some(v)),
            Token::Null => Ok(None),
            other => Err(ConversionError {
                from: "Token",
                to: "Option<u64>",
                value: format!("{:?}", other),
            }),
        }
    }
}

impl TryFrom<Token> for Option<f64> {
    type Error = ConversionError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Double(v) => Ok(Some(v)),
            Token::Null => Ok(None),
            other => Err(ConversionError {
                from: "Token",
                to: "Option<f64>",
                value: format!("{:?}", other),
            }),
        }
    }
}

impl TryFrom<Token> for Option<String> {
    type Error = ConversionError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::String(v) => Ok(Some(v)),
            Token::Null => Ok(None),
            other => Err(ConversionError {
                from: "Token",
                to: "Option<String>",
                value: format!("{:?}", other),
            }),
        }
    }
}

impl TryFrom<Token> for Option<bool> {
    type Error = ConversionError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Int(v) => Ok(Some(v != 0)),
            Token::Null => Ok(None),
            other => Err(ConversionError {
                from: "Token",
                to: "Option<bool>",
                value: format!("{:?}", other),
            }),
        }
    }
}

impl TryFrom<Token> for Option<usize> {
    type Error = ConversionError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Array(v) => Ok(Some(v)),
            Token::Null => Ok(None),
            other => Err(ConversionError {
                from: "Token",
                to: "Option<usize>",
                value: format!("{:?}", other),
            }),
        }
    }
}

impl TryFrom<Token> for Option<i32> {
    type Error = ConversionError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Int(v) => Ok(Some(v as i32)),
            Token::Null => Ok(None),
            other => Err(ConversionError {
                from: "Token",
                to: "Option<i32>",
                value: format!("{:?}", other),
            }),
        }
    }
}

impl TryFrom<Token> for Option<i8> {
    type Error = ConversionError;

    fn try_from(value: Token) -> Result<Self, Self::Error> {
        match value {
            Token::Int(v) => Ok(Some(v as i8)),
            Token::Null => Ok(None),
            other => Err(ConversionError {
                from: "Token",
                to: "Option<i8>",
                value: format!("{:?}", other),
            }),
        }
    }
}
