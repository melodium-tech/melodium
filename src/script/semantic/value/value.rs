
//! Module for Value identification and structure semantic analysis.

use super::common::Node;

use std::sync::{Arc, Weak, RwLock};
use std::convert::TryFrom;
use crate::script::error::ScriptError;
use crate::script::path::Path;
use crate::script::text::{PositionnedString, Position};
use crate::script::text::value::Value as TextValue;
use crate::executive::value::Value as ExecutiveValue;
use crate::logic::descriptor::datatype::{DataType, Structure, Type};
use crate::logic::designer::ValueDesigner;

use super::declarative_element::{DeclarativeElement, DeclarativeElementType};
use super::common::Reference;
use super::declared_parameter::DeclaredParameter;
use super::requirement::Requirement;

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
            TextValue::Array(_, a) => Self::parse_vector(&a)?,
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

        let unsigned = n.string.parse::<u128>();
        if unsigned.is_ok() {
            return Ok(ValueContent::Unsigned(unsigned.unwrap()));
        }

        let integer = n.string.parse::<i128>();
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

    pub fn make_executive_value(&self, datatype: &DataType) -> Result<ExecutiveValue, ScriptError> {

        self.content.make_executive_value(datatype)
    }

    pub fn make_designed_value(&self, datatype: &DataType) -> Result<ValueDesigner, ScriptError> {

        match &self.content {
            ValueContent::Name(decl_param) => {
                Ok(ValueDesigner::Variable(decl_param.name.clone()))
            },
            ValueContent::ContextReference((context, name)) => {
                Ok(ValueDesigner::Context((context.name.clone(), name.clone())))
            },
            _ => {
                Ok(ValueDesigner::Raw(self.make_executive_value(datatype)?))
            },
        }
    }

}

impl Node for Value {
    fn make_references(&mut self, _: &Path) -> Result<(), ScriptError> {

        let content = self.make_reference_valuecontent(&self.content)?;

        self.content = content;

        Ok(())
    }
}
