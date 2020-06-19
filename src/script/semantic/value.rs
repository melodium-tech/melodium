
use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::value::Value as TextValue;

use super::sequence::Sequence;
use super::reference::Reference;
use super::declared_parameter::DeclaredParameter;
use super::requirement::Requirement;

pub enum ValueContent {
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(String),
    Array(Vec<ValueContent>),
    Name(Reference<DeclaredParameter>),
    Reference(Reference<Requirement>)
}

pub struct Value {
    pub text: TextValue,

    pub sequence: Rc<RefCell<Sequence>>,

    pub content: ValueContent,
}

impl Value {
    pub fn new(sequence: Rc<RefCell<Sequence>>, text: TextValue) -> Result<Rc<RefCell<Self>>, ScriptError> {

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self{
            sequence,
            content: Self::parse(&text)?,
            text,
        })))
    }

    fn parse(text: &TextValue) -> Result<ValueContent, ScriptError> {

        let content = match text {
            TextValue::Boolean(b) => Self::parse_boolean(&b)?,
            TextValue::Number(n) => Self::parse_number(&n)?,
            TextValue::String(s) => Self::parse_string(&s)?,
            TextValue::Array(a) => Self::parse_vector(&a)?,
            TextValue::Name(n) => ValueContent::Name(Reference::new(n.to_string())),
            TextValue::Reference(r) => ValueContent::Reference(Reference::new(r.to_string())),
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

        let mut values = Vec::new();
        for val in v {
            values.push(Self::parse(val)?);
        }

        Ok(ValueContent::Array(values))
    }

    pub fn make_references(&mut self) -> Result<(), ScriptError> {

        let content = self.make_reference_valuecontent(&self.content)?;

        self.content = content;

        Ok(())
    }

    fn make_reference_valuecontent(&self, value: &ValueContent) -> Result<ValueContent, ScriptError> {

        let content;
        let borrowed_sequence = self.sequence.borrow();

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
                let param = borrowed_sequence.find_parameter(&n.name);
                if param.is_some() {
                    
                    content = ValueContent::Name(Reference {
                        name: n.name.clone(),
                        reference: Some(Rc::clone(param.unwrap())),
                    });
                }
                else {
                    return Err(ScriptError::semantic("Unkown name '".to_string() + &n.name + "' in sequence parameters."));
                }
            },
            ValueContent::Reference(r) => {
                let requirement = borrowed_sequence.find_requirement(&r.name);
                if requirement.is_some() {

                    content = ValueContent::Reference(Reference {
                        name: r.name.clone(),
                        reference: Some(Rc::clone(requirement.unwrap())),
                    });
                }
                else {
                    return Err(ScriptError::semantic("Unkown reference '".to_string() + &r.name + "' in sequence requirements."));
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
