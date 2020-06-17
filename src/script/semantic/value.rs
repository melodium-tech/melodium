
use crate::script::error::ScriptError;
use crate::script::text::value::Value as TextValue;

pub enum ValueContent {
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(String),
    Array(Vec<ValueContent>),
    Name(String),
    Reference(String)
}

pub struct Value {
    pub text: TextValue,

    pub content: ValueContent,
}

impl Value {
    pub fn new(text: TextValue) -> Result<Self, ScriptError> {

        Ok(Self{
            text,
            content: Self::parse(&text)?
        })
    }

    fn parse(text: &TextValue) -> Result<ValueContent, ScriptError> {

        let content = match text {
            TextValue::Boolean(b) => Self::parse_boolean(&b)?,
            TextValue::Number(n) => Self::parse_number(&n)?,
            TextValue::String(s) => Self::parse_string(&s)?,
            TextValue::Name(n) => ValueContent::Name(n.to_string()),
            TextValue::Reference(r) => ValueContent::Reference(r.to_string()),
        };

        Ok(content)
    }

    fn parse_boolean(b: &str) -> Result<ValueContent, ScriptError> {
        Ok(ValueContent::Boolean(
            if b == "true" { true }
            else if b == "false" { false }
            else {
                return Err(ScriptError::semantic("'".to_string() + &b + "' is not a valid boolean."))
            }
        ))
    }

    fn parse_number(n: &str) -> Result<ValueContent, ScriptError> {

        let integer = n.parse::<i64>();
        if integer.is_ok() {
            return Ok(ValueContent::Integer(integer.unwrap()));
        }

        let real = n.parse::<f64>();
        if real.is_ok() {
            return Ok(ValueContent::Real(real.unwrap()));
        }

        Err(ScriptError::semantic("'".to_string() + &n + "' is not a valid number."))
    }

    fn parse_string(s: &str) -> Result<ValueContent, ScriptError> {

        let string = s.strip_prefix('"');
        if string.is_none() {
            return Err(ScriptError::semantic("String not starting with '\"', this is an internal bug.".to_string()));
        }
        let string = string.unwrap().strip_suffix('"');
        if string.is_none() {
            return Err(ScriptError::semantic("String not ending with '\"', this is an internal bug.".to_string()));
        }

        let string = string.unwrap().replace(r#"\""#, r#"""#).replace(r#"\\"#, r#"\"#);

        Ok(ValueContent::String(string))
    }

    fn parse_vector(v: &Vec<TextValue>) -> Result<ValueContent, ScriptError> {

        let values = Vec::new();
        for val in v {
            values.push(Self::parse(val)?);
        }

        Ok(ValueContent::Array(values))
    }
}
