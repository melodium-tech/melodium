
//! Module for Value identification and structure semantic analysis.

use super::common::Node;

use std::sync::{Arc, Weak, RwLock};
use crate::script::error::ScriptError;
use crate::script::path::Path;
use crate::script::text::{PositionnedString, Position};
use crate::script::text::value::Value as TextValue;
use crate::executive::value::Value as ExecutiveValue;

use super::declarative_element::{DeclarativeElement, DeclarativeElementType};
use super::common::Reference;
use super::declared_parameter::DeclaredParameter;
use super::requirement::Requirement;

/// Enum holding value or reference designating the value.
pub enum ValueContent {
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(String),
    /// Array, allowing recursive values (in case of vectors).
    Array(Vec<ValueContent>),
    /// Named value, referring to a parameter of the hosting sequence.
    Name(Reference<DeclaredParameter>),
    /// Context reference, referring to a requirement of the hosting sequence, and an inner element.
    ContextReference((Reference<Requirement>, String))
}

impl ValueContent {
    pub fn make_executive_value(&self) -> Result<ExecutiveValue, ScriptError> {

        let value = match *self {
            ValueContent::Boolean(b) => Some(ExecutiveValue::Boolean(b)),
            ValueContent::Integer(i) => Some(ExecutiveValue::Integer(i)),
            ValueContent::Real(r) => Some(ExecutiveValue::Real(r)),
            ValueContent::String(s) => Some(ExecutiveValue::String(s)),
            ValueContent::Array(a) => {

                let exp_val_type;
                if let Some(first_val) = a.first() {
                    exp_val_type = first_val.make_executive_value()?;

                    match exp_val_type {
                        ExecutiveValue::Boolean(_) => {
                            let vec = a.iter().map(|val| match val{ValueContent::Boolean(b)=>*b}).collect();
                            Some(ExecutiveValue::VecBoolean(vec))
                        },
                        ExecutiveValue::Integer(_) => {
                            let vec = a.iter().map(|val| match val{ValueContent::Integer(i)=>*i}).collect();
                            Some(ExecutiveValue::VecInteger(vec))
                        },
                        ExecutiveValue::Real(_) => {
                            let vec = a.iter().map(|val| match val{ValueContent::Real(r)=>*r}).collect();
                            Some(ExecutiveValue::VecReal(vec))
                        },
                        ExecutiveValue::String(_) => {
                            let vec = a.iter().map(|val| match val{ValueContent::String(s)=>*s}).collect();
                            Some(ExecutiveValue::VecString(vec))
                        },
                        _ => None
                    }
                }
                else {
                    // TODO
                    panic!("Empty array, implement management");
                }
            },
            _ => None
        };

        if value.is_some() {
            Ok(value.unwrap())
        }
        else {
            // TODO
            panic!("Wrong value typing, implement management");
        }
    }
}

/// Structure managing and describing Value semantic analysis.
/// 
/// It owns the whole [text value](../../text/value/enum.Value.html).
/// A reference to the declarative element it belongs to is needed for cases the value is or contains a name or reference.
pub struct Value {
    pub text: TextValue,

    pub host: Weak<RwLock<dyn DeclarativeElement>>,

    pub content: ValueContent,
}

impl Value {
    /// Create a new semantic value, based on textual value.
    /// 
    /// * `host`: the declarative element that host the value.
    /// * `text`: the textual value.
    /// 
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](../common/trait.Node.html).
    pub fn new(host: Arc<RwLock<dyn DeclarativeElement>>, text: TextValue) -> Result<Arc<RwLock<Self>>, ScriptError> {

        Ok(Arc::<RwLock<Self>>::new(RwLock::new(Self{
            host: Arc::downgrade(&host),
            content: Self::parse(&text)?,
            text,
        })))
    }

    fn parse(text: &TextValue) -> Result<ValueContent, ScriptError> {

        let content = match text {
            TextValue::Boolean(b) => Self::parse_boolean(b)?,
            TextValue::Number(n) => Self::parse_number(n)?,
            TextValue::String(s) => Self::parse_string(s)?,
            TextValue::Array(a) => Self::parse_vector(&a)?,
            TextValue::Name(n) => ValueContent::Name(Reference::new(n.string.to_string())),
            TextValue::ContextReference((r, e)) => ValueContent::ContextReference((Reference::new(r.string.to_string()), e.string.to_string())),
        };

        Ok(content)
    }

    fn parse_boolean(b: & PositionnedString) -> Result<ValueContent, ScriptError> {
        Ok(ValueContent::Boolean(
            if b.string == "true" { true }
            else if b.string == "false" { false }
            else {
                return Err(ScriptError::semantic("'".to_string() + &b.string + "' is not a valid boolean.", b.position))
            }
        ))
    }

    fn parse_number(n: & PositionnedString) -> Result<ValueContent, ScriptError> {

        let integer = n.string.parse::<i64>();
        if integer.is_ok() {
            return Ok(ValueContent::Integer(integer.unwrap()));
        }

        let real = n.string.parse::<f64>();
        if real.is_ok() {
            return Ok(ValueContent::Real(real.unwrap()));
        }

        Err(ScriptError::semantic("'".to_string() + &n.string + "' is not a valid number.", n.position))
    }

    fn parse_string(s: & PositionnedString) -> Result<ValueContent, ScriptError> {

        let string = s.string.strip_prefix('"');
        if string.is_none() {
            return Err(ScriptError::semantic("String not starting with '\"', this is an internal bug.".to_string(), s.position));
        }
        let string = string.unwrap().strip_suffix('"');
        if string.is_none() {
            return Err(ScriptError::semantic("String not ending with '\"', this is an internal bug.".to_string(), s.position));
        }

        let string = string.unwrap().replace(r#"\""#, r#"""#).replace(r#"\\"#, r#"\"#);

        Ok(ValueContent::String(string))
    }

    fn parse_vector(v: &Vec<TextValue>) -> Result<ValueContent, ScriptError> {

        let mut values = Vec::new();
        for val in v {
            values.push(Self::parse(val)?);
        }

        Ok(ValueContent::Array(values))
    }

    fn make_reference_valuecontent(&self, value: &ValueContent) -> Result<ValueContent, ScriptError> {

        let rc_host = self.host.upgrade().unwrap();
        let borrowed_host = rc_host.read().unwrap();
        let content;

        match value {
            ValueContent::Boolean(b) => {
                content = ValueContent::Boolean(*b);
            },
            ValueContent::Integer(i) => {
                content = ValueContent::Integer(*i);
            },
            ValueContent::Real(r) => {
                content = ValueContent::Real(*r);
            },
            ValueContent::String(s) => {
                content = ValueContent::String(s.clone());
            },
            ValueContent::Name(n) => {

                let param = borrowed_host.find_declared_parameter(&n.name);
                if param.is_some() {
                    
                    content = ValueContent::Name(Reference {
                        name: n.name.clone(),
                        reference: Some(Arc::downgrade(&param.unwrap())),
                    });
                }
                else {
                    let position = match &self.text {
                        TextValue::Name(ps) => ps.position,
                        _ => Position::default(),
                    };
                    return Err(ScriptError::semantic("Unkown name '".to_string() + &n.name + "' in declared parameters.", position));
                }
            },
            ValueContent::ContextReference((r, e)) => {

                let requirement = match &borrowed_host.declarative_element() {
                    DeclarativeElementType::Sequence(s) => s.find_requirement(&r.name),
                    _ => None,
                };

                if requirement.is_some() {

                    content = ValueContent::ContextReference((Reference {
                        name: r.name.clone(),
                        reference: Some(Arc::downgrade(&requirement.unwrap())),
                    }, e.clone()));
                }
                else {
                    let position = match &self.text {
                        TextValue::ContextReference((ps, _)) => ps.position,
                        _ => Position::default(),
                    };
                    return Err(ScriptError::semantic("Unkown context '".to_string() + &r.name + "' in sequence requirements.", position));
                }
            },
            ValueContent::Array(a) => {

                let mut array = Vec::new();
                for v in a {
                    array.push(self.make_reference_valuecontent(v)?);
                }

                content = ValueContent::Array(array);
            },
        }

        Ok(content)
    }

}

impl Node for Value {
    fn make_references(&mut self, path: &Path) -> Result<(), ScriptError> {

        let content = self.make_reference_valuecontent(&self.content)?;

        self.content = content;

        Ok(())
    }
}
