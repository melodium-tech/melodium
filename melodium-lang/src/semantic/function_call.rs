//! Module dedicated to Treatment semantic analysis.

use super::common::Node;
use super::common::Reference;
use super::declarative_element::{DeclarativeElement, DeclarativeElementType};
use super::r#use::Use;
use super::value::Value;
use crate::error::ScriptError;
use crate::path::Path;
use crate::text::Function as TextFunction;
use melodium_common::descriptor::identifier::Identifier;
use std::sync::{Arc, RwLock, Weak};

#[derive(Debug)]
pub enum RefersTo {
    Unknown(Reference<()>),
    Use(Reference<Use>),
}

/// Structure managing and describing semantic of a function call.
///
#[derive(Debug)]
pub struct FunctionCall {
    pub text: TextFunction,

    pub scope: Weak<RwLock<dyn DeclarativeElement>>,

    pub name: String,
    pub r#type: RefersTo,
    pub parameters: Vec<Arc<RwLock<Value>>>,

    pub type_identifier: Option<Identifier>,
}

impl FunctionCall {
    pub fn new(
        scope: Arc<RwLock<dyn DeclarativeElement>>,
        text: TextFunction,
    ) -> Result<Arc<RwLock<Self>>, ScriptError> {
        let mut parameters = Vec::new();
        for val in &text.parameters {
            let value = Value::new(
                Arc::clone(&scope) as Arc<RwLock<dyn DeclarativeElement>>,
                val.clone(),
            )?;

            parameters.push(value);
        }

        Ok(Arc::<RwLock<Self>>::new(RwLock::new(Self {
            name: text.name.string.clone(),
            r#type: RefersTo::Unknown(Reference::new(text.name.string.clone())),
            scope: Arc::downgrade(&scope),
            text,
            parameters,
            type_identifier: None,
        })))
    }
}

impl Node for FunctionCall {
    fn make_references(&mut self, _path: &Path) -> Result<(), ScriptError> {
        if let RefersTo::Unknown(reference) = &self.r#type {
            let rc_script = match self
                .scope
                .upgrade()
                .unwrap()
                .read()
                .unwrap()
                .declarative_element()
            {
                DeclarativeElementType::Model(m) => m.script.upgrade().unwrap(),
                DeclarativeElementType::Treatment(t) => t.script.upgrade().unwrap(),
            };
            let borrowed_script = rc_script.read().unwrap();

            if let Some(r#use) = borrowed_script.find_use(&reference.name) {
                self.type_identifier = r#use.read().unwrap().identifier.clone();

                self.r#type = RefersTo::Use(Reference {
                    name: reference.name.clone(),
                    reference: Some(Arc::downgrade(r#use)),
                });
            }
            // Add here when and if functions can be scripted to found them in local script file.
            else {
                return Err(ScriptError::semantic(
                    format!("'{}' is unknown.", self.text.name.string),
                    self.text.name.position,
                ));
            }
        }

        Ok(())
    }

    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {
        let mut children: Vec<Arc<RwLock<dyn Node>>> = Vec::new();

        for value in &self.parameters {
            children.push(Arc::clone(&value) as Arc<RwLock<dyn Node>>);
        }

        children
    }
}
