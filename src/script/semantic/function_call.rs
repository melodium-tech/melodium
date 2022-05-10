
//! Module dedicated to Treatment semantic analysis.

use super::common::Node;

use std::sync::{Arc, Weak, RwLock};
use crate::script::error::{ScriptError, wrap_logic_error};
use crate::script::text::Function as TextFunction;

use super::r#use::Use;
use super::declarative_element::DeclarativeElement;
use super::value::Value;

/// Structure managing and describing semantic of a function call.
/// 
#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub text: TextFunction,

    pub scope: Weak<RwLock<dyn DeclarativeElement>>,

    pub name: String,
    pub parameters: Vec<Arc<RwLock<Value>>>,
}

impl FunctionCall {

    pub fn new(scope: Arc<RwLock<dyn DeclarativeElement>>, text: TextFunction) -> Result<Arc<RwLock<Self>>, ScriptError> {

        let mut parameters = Vec::new();
        for val in &text.parameters {
            let value = Value::new(Arc::clone(&scope) as Arc<RwLock<dyn DeclarativeElement>>, val.clone())?;

            parameters.push(value);
        }

        Ok(Arc::<RwLock<Self>>::new(RwLock::new(Self {
            name: text.name.string.clone(),
            text,
            scope: Arc::downgrade(&scope),
            parameters,
        })))
    }
}

impl Node for FunctionCall {
    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {

        let mut children: Vec<Arc<RwLock<dyn Node>>> = Vec::new();

        for value in &self.parameters {
            children.push(Arc::clone(&value) as Arc<RwLock<dyn Node>>);
        }

        children
    }
}
