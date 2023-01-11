
//! Module for Value identification and structure semantic analysis.

use super::super::common::Node;

use std::sync::{Arc, Weak, RwLock};
use crate::script::error::ScriptError;
use crate::script::path::Path;
use crate::script::text::{PositionnedString, Position};
use crate::script::text::value::Value as TextValue;
use crate::executive::value::Value as ExecutiveValue;
use crate::logic::descriptor::datatype::DataType;
use crate::logic::designer::ValueDesigner;
use crate::logic::designer::ParameterDesigner;

use super::ValueContent;
use super::super::declarative_element::{DeclarativeElement, DeclarativeElementType};
use super::super::common::Reference;
use super::super::function_call::FunctionCall;

/// Structure managing and describing Value semantic analysis.
/// 
/// It owns the whole [text value](../../text/value/enum.Value.html).
/// A reference to the declarative element it belongs to is needed for cases the value is or contains a name or reference.
#[derive(Debug)]
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
            content: Self::parse(host, &text)?,
            text,
        })))
    }

    fn parse(host: Arc<RwLock<dyn DeclarativeElement>>, text: &TextValue) -> Result<ValueContent, ScriptError> {

        let content = match text {
            TextValue::Boolean(b) => Self::parse_boolean(b)?,
            TextValue::Number(n) => Self::parse_number(n)?,
            TextValue::String(s) => Self::parse_string(s)?,
            TextValue::Character(c) => Self::parse_character(c)?,
            TextValue::Byte(b) => Self::parse_byte(b)?,
            TextValue::Array(_, a) => Self::parse_vector(host, &a)?,
            TextValue::Name(n) => ValueContent::Name(Reference::new(n.string.to_string())),
            TextValue::ContextReference((r, e)) => ValueContent::ContextReference((Reference::new(r.string.to_string()), e.string.to_string())),
            TextValue::Function(f) => ValueContent::Function(FunctionCall::new(host, f.clone())?),
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

    fn parse_character(c: & PositionnedString) -> Result<ValueContent, ScriptError> {

        if let Some(character) = c.string.strip_prefix('\'') {
            if let Some(character) = character.strip_suffix('\'') {
                Ok(ValueContent::Character(character.chars().next().unwrap()))
            }
            else {
                Err(ScriptError::semantic("Character not ending with ''', this is an internal bug.".to_string(), c.position))
            }
        }
        else {
            Err(ScriptError::semantic("Character not starting with ''', this is an internal bug.".to_string(), c.position))
        }        
    }

    fn parse_byte(b: & PositionnedString) -> Result<ValueContent, ScriptError> {

        if let Some(byte) = b.string.strip_prefix("0x") {
            if let Ok(byte) = hex::decode(byte) {
                Ok(ValueContent::Byte(byte[0]))
            }
            else {
                Err(ScriptError::semantic("Byte notation not valid as hexadecimal, this is an internal bug.".to_string(), b.position))
            }
        }
        else {
            Err(ScriptError::semantic("Byte notation not starting with '0x', this is an internal bug.".to_string(), b.position))
        }        
    }

    fn parse_vector(host: Arc<RwLock<dyn DeclarativeElement>>, v: &Vec<TextValue>) -> Result<ValueContent, ScriptError> {

        let mut values = Vec::new();
        for val in v {
            values.push(Self::parse(Arc::clone(&host), val)?);
        }

        Ok(ValueContent::Array(values))
    }

    fn make_reference_valuecontent(&self, value: &ValueContent, path: &Path) -> Result<ValueContent, ScriptError> {

        let rc_host = self.host.upgrade().unwrap();
        let borrowed_host = rc_host.read().unwrap();
        let content;

        match value {
            ValueContent::Boolean(b) => {
                content = ValueContent::Boolean(*b);
            },
            ValueContent::Unsigned(u) => {
                content = ValueContent::Unsigned(*u);
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
            ValueContent::Character(c) => {
                content = ValueContent::Character(*c);
            },
            ValueContent::Byte(b) => {
                content = ValueContent::Byte(*b);
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
            ValueContent::Function(f) => {

                f.write().unwrap().make_references(path)?;

                content = ValueContent::Function(f.clone());
            },
            ValueContent::Array(a) => {

                let mut array = Vec::new();
                for v in a {
                    array.push(self.make_reference_valuecontent(v, path)?);
                }

                content = ValueContent::Array(array);
            },
        }

        Ok(content)
    }

    pub fn make_executive_value(&self, datatype: &DataType) -> Result<ExecutiveValue, ScriptError> {

        let possible_value = self.content.make_executive_value(datatype);

        if possible_value.is_ok() {
            Ok(possible_value.unwrap())
        }
        else {
            Err(ScriptError::semantic(possible_value.unwrap_err(), self.text.get_position()))
        }
    }

    pub fn make_designed_value(&self, designer: &ParameterDesigner, datatype: &DataType) -> Result<ValueDesigner, ScriptError> {

        match &self.content {
            ValueContent::Name(decl_param) => {
                Ok(ValueDesigner::Variable(decl_param.name.clone()))
            },
            ValueContent::ContextReference((context, name)) => {
                Ok(ValueDesigner::Context((context.name.clone(), name.clone())))
            },
            ValueContent::Function(func) => {
                let borrowed_func = func.read().unwrap();

                if let Some(func_descriptor) = designer.scope().upgrade().unwrap()
                                                            .read().unwrap()
                                                            .collections().functions.get(
                                                                &borrowed_func.type_identifier.as_ref().unwrap()
                ) {

                    let mut params = Vec::new();
                    for i in 0..func_descriptor.parameters().len() {

                        let desc_param = &func_descriptor.parameters()[i];

                        if let Some(rc_param) = borrowed_func.parameters.get(i) {
                            let borrowed_param = rc_param.read().unwrap();

                            let param = borrowed_param.make_designed_value(designer, desc_param.datatype())?;

                            params.push(param);
                        }
                        else {
                            return Err(ScriptError::semantic(format!("Missing parameter on function {}.", borrowed_func.name), self.text.get_position()))
                        }
                    }

                    Ok(ValueDesigner::Function(Arc::clone(func_descriptor), params))
                }
                else {
                    Err(ScriptError::semantic(format!("No known function '{}'.", borrowed_func.name), self.text.get_position()))
                }
            },
            _ => {
                Ok(ValueDesigner::Raw(self.make_executive_value(datatype)?))
            },
        }
    }

}

impl Node for Value {
    fn make_references(&mut self, path: &Path) -> Result<(), ScriptError> {

        let content = self.make_reference_valuecontent(&self.content, path)?;

        self.content = content;

        Ok(())
    }

    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {

        let mut children: Vec<Arc<RwLock<dyn Node>>> = Vec::new();

        if let ValueContent::Function(f) = &self.content {
            children.extend(f.read().unwrap().children());
        }

        children
    }
}
