//! Module for Value identification and structure semantic analysis.

use super::super::common::Node;
use super::super::common::Reference;
use super::super::declarative_element::{DeclarativeElement, DeclarativeElementType};
use super::super::function_call::FunctionCall;
use super::ValueContent;
use crate::error::ScriptError;
use crate::path::Path;
use crate::text::value::Value as TextValue;
use crate::text::PositionnedString;
use crate::ScriptResult;
use melodium_common::descriptor::Collection;
use melodium_common::descriptor::DescribedType;
use melodium_common::descriptor::VersionReq;
use melodium_common::descriptor::{DataType, Entry};
use melodium_common::executive::Value as ExecutiveValue;
use melodium_engine::designer::{Parameter as ParameterDesigner, Value as ValueDesigner};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing Value semantic analysis.
///
/// It owns the whole [text value](TextValue).
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
    pub fn new(
        host: Arc<RwLock<dyn DeclarativeElement>>,
        text: TextValue,
    ) -> ScriptResult<Arc<RwLock<Self>>> {
        Self::parse(host.clone(), &text).and_then(|value_content| {
            ScriptResult::new_success(Arc::<RwLock<Self>>::new(RwLock::new(Self {
                host: Arc::downgrade(&host),
                content: value_content,
                text,
            })))
        })
    }

    fn parse(
        host: Arc<RwLock<dyn DeclarativeElement>>,
        text: &TextValue,
    ) -> ScriptResult<ValueContent> {
        match text {
            TextValue::Void(_) => ScriptResult::new_success(ValueContent::Void),
            TextValue::Boolean(b) => Self::parse_boolean(b),
            TextValue::Number(n) => Self::parse_number(n),
            TextValue::String(s) => Self::parse_string(s),
            TextValue::Character(c) => Self::parse_character(c),
            TextValue::Byte(b) => Self::parse_byte(b),
            TextValue::Array(_, a) => Self::parse_vector(host, &a),
            TextValue::Name(n) => {
                ScriptResult::new_success(ValueContent::Name(Reference::new(n.string.to_string())))
            }
            TextValue::ContextReference((r, e)) => {
                ScriptResult::new_success(ValueContent::ContextReference((
                    Reference::new(r.string.to_string()),
                    e.string.to_string(),
                )))
            }
            TextValue::Function(f) => FunctionCall::new(host, f.clone())
                .and_then(|func| ScriptResult::new_success(ValueContent::Function(func))),
        }
    }

    fn parse_boolean(b: &PositionnedString) -> ScriptResult<ValueContent> {
        if b.string == "true" {
            ScriptResult::new_success(ValueContent::Boolean(true))
        } else if b.string == "false" {
            ScriptResult::new_success(ValueContent::Boolean(false))
        } else {
            ScriptResult::new_failure(ScriptError::invalid_boolean(144, b.clone()))
        }
    }

    fn parse_number(n: &PositionnedString) -> ScriptResult<ValueContent> {
        let unsigned = n.string.parse::<u128>();
        if unsigned.is_ok() {
            return ScriptResult::new_success(ValueContent::Unsigned(unsigned.unwrap()));
        }

        let integer = n.string.parse::<i128>();
        if integer.is_ok() {
            return ScriptResult::new_success(ValueContent::Integer(integer.unwrap()));
        }

        let real = n.string.parse::<f64>();
        if real.is_ok() {
            return ScriptResult::new_success(ValueContent::Real(real.unwrap()));
        }

        ScriptResult::new_failure(ScriptError::invalid_number(145, n.clone()))
    }

    fn parse_string(s: &PositionnedString) -> ScriptResult<ValueContent> {
        let string = s.string.strip_prefix('"');
        if string.is_none() {
            return ScriptResult::new_failure(ScriptError::invalid_string(146, s.clone()));
        }
        let string = string.unwrap().strip_suffix('"');
        if string.is_none() {
            return ScriptResult::new_failure(ScriptError::invalid_string(147, s.clone()));
        }

        let string = string
            .unwrap()
            .replace(r#"\""#, r#"""#)
            .replace(r#"\\"#, r#"\"#);

        ScriptResult::new_success(ValueContent::String(string))
    }

    fn parse_character(c: &PositionnedString) -> ScriptResult<ValueContent> {
        if let Some(character) = c.string.strip_prefix('\'') {
            if let Some(character) = character.strip_suffix('\'') {
                ScriptResult::new_success(ValueContent::Character(
                    character.chars().next().unwrap(),
                ))
            } else {
                ScriptResult::new_failure(ScriptError::invalid_character(148, c.clone()))
            }
        } else {
            ScriptResult::new_failure(ScriptError::invalid_character(149, c.clone()))
        }
    }

    fn parse_byte(b: &PositionnedString) -> ScriptResult<ValueContent> {
        if let Some(byte) = b.string.strip_prefix("0x") {
            if let Ok(byte) = hex::decode(byte) {
                ScriptResult::new_success(ValueContent::Byte(byte[0]))
            } else {
                ScriptResult::new_failure(ScriptError::invalid_character(150, b.clone()))
            }
        } else {
            ScriptResult::new_failure(ScriptError::invalid_character(151, b.clone()))
        }
    }

    fn parse_vector(
        host: Arc<RwLock<dyn DeclarativeElement>>,
        v: &Vec<TextValue>,
    ) -> ScriptResult<ValueContent> {
        let mut result = ScriptResult::new_success(());
        let mut values = Vec::new();
        for val in v {
            if let Some(val) = result.merge_degrade_failure(Self::parse(Arc::clone(&host), val)) {
                values.push(val);
            }
        }

        result.and_then(|_| ScriptResult::new_success(ValueContent::Array(values)))
    }

    fn make_reference_valuecontent(
        &self,
        value: &ValueContent,
        path: &Path,
        versions: &HashMap<String, VersionReq>,
    ) -> ScriptResult<ValueContent> {
        let rc_host = self.host.upgrade().unwrap();
        let borrowed_host = rc_host.read().unwrap();
        let content;

        match value {
            ValueContent::Void => {
                content = ValueContent::Void;
            }
            ValueContent::Boolean(b) => {
                content = ValueContent::Boolean(*b);
            }
            ValueContent::Unsigned(u) => {
                content = ValueContent::Unsigned(*u);
            }
            ValueContent::Integer(i) => {
                content = ValueContent::Integer(*i);
            }
            ValueContent::Real(r) => {
                content = ValueContent::Real(*r);
            }
            ValueContent::String(s) => {
                content = ValueContent::String(s.clone());
            }
            ValueContent::Character(c) => {
                content = ValueContent::Character(*c);
            }
            ValueContent::Byte(b) => {
                content = ValueContent::Byte(*b);
            }
            ValueContent::Name(n) => {
                let param = borrowed_host.find_declared_parameter(&n.name);
                if param.is_some() {
                    content = ValueContent::Name(Reference {
                        name: n.name.clone(),
                        reference: Some(Arc::downgrade(&param.unwrap())),
                    });
                } else {
                    let ps = match &self.text {
                        TextValue::Name(ps) => ps.clone(),
                        _ => PositionnedString::default(),
                    };
                    return ScriptResult::new_failure(ScriptError::undeclared_parameter(152, ps));
                }
            }
            ValueContent::ContextReference((r, e)) => {
                let requirement = match &borrowed_host.declarative_element() {
                    DeclarativeElementType::Treatment(s) => s.find_requirement(&r.name),
                    _ => None,
                };

                if requirement.is_some() {
                    content = ValueContent::ContextReference((
                        Reference {
                            name: r.name.clone(),
                            reference: Some(Arc::downgrade(&requirement.unwrap())),
                        },
                        e.clone(),
                    ));
                } else {
                    let ps = match &self.text {
                        TextValue::ContextReference((ps, _)) => ps.clone(),
                        _ => PositionnedString::default(),
                    };
                    return ScriptResult::new_failure(ScriptError::undeclared_context(153, ps));
                }
            }
            ValueContent::Function(f) => {
                return f
                    .write()
                    .unwrap()
                    .make_references(path, versions)
                    .and_then(|_| ScriptResult::new_success(ValueContent::Function(f.clone())))
            }
            ValueContent::Array(a) => {
                let mut result = ScriptResult::new_success(());
                let mut array = Vec::new();
                for v in a {
                    if let Some(val) = result
                        .merge_degrade_failure(self.make_reference_valuecontent(v, path, versions))
                    {
                        array.push(val);
                    }
                }

                return result.and_then(|_| ScriptResult::new_success(ValueContent::Array(array)));
            }
        }

        ScriptResult::new_success(content)
    }

    pub fn make_executive_value(&self, datatype: &DataType) -> ScriptResult<ExecutiveValue> {
        match self.content.make_executive_value(datatype) {
            Ok(value) => ScriptResult::new_success(value),
            Err(err) => ScriptResult::new_failure(ScriptError::executive_restitution_failed(
                154,
                self.text.get_positionned_string().clone(),
                err,
            )),
        }
    }

    pub fn make_designed_value(
        &self,
        designer: &ParameterDesigner,
        described_type: &DescribedType,
        collection: &Collection,
    ) -> ScriptResult<ValueDesigner> {
        Self::build_designed_value(
            &self.content,
            designer,
            described_type,
            collection,
            self.text.get_positionned_string(),
        )
    }

    fn build_designed_value(
        content: &ValueContent,
        designer: &ParameterDesigner,
        described_type: &DescribedType,
        collection: &Collection,
        positioned_string: &PositionnedString,
    ) -> ScriptResult<ValueDesigner> {
        match content {
            ValueContent::Name(decl_param) => {
                ScriptResult::new_success(ValueDesigner::Variable(decl_param.name.clone()))
            }
            ValueContent::ContextReference((context, name)) => {
                if let Some(Entry::Context(context)) = designer
                    .scope()
                    .upgrade()
                    .unwrap()
                    .read()
                    .unwrap()
                    .collection()
                    .get(
                        &context
                            .reference
                            .as_ref()
                            .unwrap()
                            .upgrade()
                            .unwrap()
                            .read()
                            .unwrap()
                            .type_identifier
                            .as_ref()
                            .unwrap(),
                    )
                {
                    ScriptResult::new_success(ValueDesigner::Context(
                        Arc::clone(context),
                        name.clone(),
                    ))
                } else {
                    ScriptResult::new_failure(ScriptError::undeclared_context(
                        155,
                        positioned_string.clone(),
                    ))
                }
            }
            ValueContent::Function(func) => {
                let borrowed_func = func.read().unwrap();

                if let Some(Entry::Function(func_descriptor)) = designer
                    .scope()
                    .upgrade()
                    .unwrap()
                    .read()
                    .unwrap()
                    .collection()
                    .get(&borrowed_func.type_identifier.as_ref().unwrap())
                {
                    let mut result = ScriptResult::new_success(());
                    let mut generics = HashMap::new();
                    let mut params = Vec::new();
                    for i in 0..func_descriptor.generics().len() {
                        let desc_generic = &func_descriptor.generics()[i];

                        if let Some(rc_generic) = borrowed_func.generics.get(i) {
                            let borrowed_generic = rc_generic.read().unwrap();
                            let borrowed_type = borrowed_generic.r#type.read().unwrap();

                            if let Some((r#type, _)) = result
                                .merge_degrade_failure(borrowed_type.make_descriptor(collection))
                            {
                                generics.insert(desc_generic.name.clone(), r#type);
                            }
                        } else {
                            result = result.and_degrade_failure(ScriptResult::new_failure(
                                ScriptError::missing_function_generic(
                                    171,
                                    positioned_string.clone(),
                                    i,
                                ),
                            ));
                        }
                    }
                    for i in 0..func_descriptor.parameters().len() {
                        let desc_param = &func_descriptor.parameters()[i];

                        if let Some(rc_param) = borrowed_func.parameters.get(i) {
                            let borrowed_param = rc_param.read().unwrap();

                            let described_type = desc_param
                                .described_type()
                                .as_defined(&generics)
                                .unwrap_or_else(|| desc_param.described_type().clone());

                            if let Some(param) =
                                result.merge_degrade_failure(borrowed_param.make_designed_value(
                                    designer,
                                    &described_type,
                                    collection,
                                ))
                            {
                                params.push(param);
                            }
                        } else {
                            result = result.and_degrade_failure(ScriptResult::new_failure(
                                ScriptError::missing_function_parameter(
                                    156,
                                    positioned_string.clone(),
                                    i,
                                ),
                            ));
                        }
                    }

                    result.and_then(|_| {
                        ScriptResult::new_success(ValueDesigner::Function(
                            Arc::clone(func_descriptor),
                            generics,
                            params,
                        ))
                    })
                } else {
                    ScriptResult::new_failure(ScriptError::unimported_element(
                        157,
                        positioned_string.clone(),
                    ))
                }
            }
            ValueContent::Array(array) => {
                if let DescribedType::Vec(inner_type) = described_type {
                    let mut vector = Vec::with_capacity(array.len());
                    for val in array {
                        let val = Self::build_designed_value(
                            val,
                            designer,
                            &inner_type,
                            collection,
                            positioned_string,
                        );
                        if val.is_failure() {
                            return val;
                        } else {
                            match val {
                                melodium_common::descriptor::Status::Success {
                                    success,
                                    errors: _,
                                } => vector.push(success),
                                melodium_common::descriptor::Status::Failure {
                                    failure: _,
                                    errors: _,
                                } => unreachable!(),
                            }
                        }
                    }
                    ScriptResult::new_success(ValueDesigner::Array(vector))
                } else {
                    ScriptResult::new_failure(ScriptError::invalid_type(
                        182,
                        positioned_string.clone(),
                    ))
                }
            }
            value => described_type
                .to_datatype(&HashMap::new())
                .map(|datatype| match value.make_executive_value(&datatype) {
                    Ok(val) => ScriptResult::new_success(ValueDesigner::Raw(val)),
                    Err(err) => {
                        ScriptResult::new_failure(ScriptError::executive_restitution_failed(
                            183,
                            positioned_string.clone(),
                            err,
                        ))
                    }
                })
                .unwrap_or_else(|| {
                    ScriptResult::new_failure(ScriptError::invalid_type(
                        109,
                        positioned_string.clone(),
                    ))
                }),
        }
    }
}

impl Node for Value {
    fn make_references(
        &mut self,
        path: &Path,
        versions: &HashMap<String, VersionReq>,
    ) -> ScriptResult<()> {
        self.make_reference_valuecontent(&self.content, path, versions)
            .and_then(|content| {
                self.content = content;
                ScriptResult::new_success(())
            })
    }

    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {
        let mut children: Vec<Arc<RwLock<dyn Node>>> = Vec::new();

        if let ValueContent::Function(f) = &self.content {
            children.extend(f.read().unwrap().children());
        }

        children
    }
}
