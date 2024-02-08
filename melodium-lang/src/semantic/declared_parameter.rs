//! Module dedicated to DeclaredParameter semantic analysis.

use super::common::Node;
use super::declarative_element::{DeclarativeElement, DeclarativeElementType};
use super::r#type::Type;
use super::value::Value;
use super::variability::Variability;
use crate::error::ScriptError;
use crate::text::Parameter as TextParameter;
use crate::ScriptResult;
use melodium_common::descriptor::{
    Collection, Flow as FlowDescriptor, Parameter as ParameterDescriptor,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of a declared parameter.
///
/// A _declared_ parameter is a parameter for which name and type are expected, as well as an optionnal value.
/// It is used by [Treatments](super::Treatment) and [Models](super::Model).
///
/// It owns the whole [text parameter](crate::text::Parameter).
#[derive(Debug)]
pub struct DeclaredParameter {
    pub text: TextParameter,

    pub parent: Weak<RwLock<dyn DeclarativeElement>>,

    pub name: String,
    pub variability: Variability,
    pub r#type: Arc<RwLock<Type>>,
    pub value: Option<Arc<RwLock<Value>>>,
}

impl DeclaredParameter {
    /// Create a new semantic declared parameter, based on textual parameter.
    ///
    /// * `parent`: the parent element owning this declared parameter.
    /// * `text`: the textual parameter.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    ///
    pub fn new(
        parent: Arc<RwLock<dyn DeclarativeElement>>,
        text: TextParameter,
    ) -> ScriptResult<Arc<RwLock<Self>>> {
        let mut result = ScriptResult::new_success(());

        let borrowed_parent = parent.read().unwrap();

        let parameter = borrowed_parent.find_declared_parameter(&text.name.string);
        if parameter.is_some() {
            result = result.and_degrade_failure(ScriptResult::new_failure(
                ScriptError::already_declared(134, text.name.clone()),
            ));
        }

        let variability;
        match borrowed_parent.declarative_element() {
            DeclarativeElementType::Model(_) => {
                if let Some(text_variability) = &text.variability {
                    let variability = Variability::from_string(&text_variability.string).unwrap();
                    if variability != Variability::Const {
                        result = result.and_degrade_failure(ScriptResult::new_failure(
                            ScriptError::const_declaration_only(135, text.name.clone()),
                        ));
                    }
                }
                variability = Variability::Const;
            }
            DeclarativeElementType::Treatment(_) => {
                if let Some(text_variability) = &text.variability {
                    variability = Variability::from_string(&text_variability.string).unwrap();
                } else {
                    variability = Variability::Var;
                }
            }
        }

        let value = if let Some(value) = text.value.as_ref().cloned() {
            result.merge_degrade_failure(Value::new(Arc::clone(&parent), value))
        } else {
            None
        };

        if let Some(text_type) = text.r#type.clone() {
            result
                .and_degrade_failure(Type::new(Arc::clone(&parent), text_type))
                .and_then(|r#type| {
                    ScriptResult::new_success(Arc::<RwLock<Self>>::new(RwLock::new(Self {
                        parent: Arc::downgrade(&parent),
                        name: text.name.string.clone(),
                        text,
                        variability,
                        r#type: Arc::new(RwLock::new(r#type)),
                        value,
                    })))
                })
        } else {
            result.and_degrade_failure(ScriptResult::new_failure(ScriptError::missing_type(
                136,
                text.name.clone(),
            )))
        }
    }

    pub fn make_descriptor(&self, collection: &Collection) -> ScriptResult<ParameterDescriptor> {
        self.r#type
            .read()
            .unwrap()
            .make_descriptor(collection)
            .and_then(|(datatype, flow)| {
                if flow != FlowDescriptor::Block {
                    ScriptResult::new_failure(ScriptError::flow_forbidden(
                        137,
                        self.text.name.clone(),
                    ))
                } else {
                    ScriptResult::new_success((datatype, flow))
                }
            })
            .and_then(|(described_type, flow)| {
                if let Some(val) = &self.value {
                    if let Some(datatype) = described_type.to_datatype(&HashMap::new()) {
                        val.read()
                            .unwrap()
                            .make_executive_value(&datatype)
                            .and_then(|val| {
                                ScriptResult::new_success((described_type, flow, Some(val)))
                            })
                    } else {
                        ScriptResult::new_failure(ScriptError::default_forbidden(
                            172,
                            self.text.name.clone(),
                        ))
                    }
                } else {
                    ScriptResult::new_success((described_type, flow, None))
                }
            })
            .and_then(|(described_type, _, value)| {
                ScriptResult::new_success(ParameterDescriptor::new(
                    &self.name,
                    self.variability.to_descriptor(),
                    described_type,
                    value,
                    self.text
                        .annotations
                        .as_ref()
                        .map(|annotations| {
                            annotations
                                .annotations
                                .iter()
                                .filter_map(|annotation| annotation.as_attribute())
                                .collect()
                        })
                        .unwrap_or_default(),
                ))
            })
    }
}

impl Node for DeclaredParameter {
    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {
        vec![Arc::clone(&self.r#type) as Arc<RwLock<dyn Node>>]
    }
}
