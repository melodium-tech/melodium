
use std::fmt::*;
use std::sync::{Arc, Weak};
use super::identified::Identified;
use super::identifier::Identifier;
use super::documented::Documented;
use super::function::Function;
use super::ordered_parameterized::OrderedParameterized;
use super::parameter::Parameter;
use super::datatype::DataType;
use crate::executive::value::Value;

#[derive(Debug)]
pub struct CoreFunction {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    parameters: Vec<Parameter>,
    return_type: DataType,
    function: fn(Vec<Value>) -> Value,
    auto_reference: Weak<Self>,
}

impl CoreFunction {
    pub fn new(
        identifier: Identifier,
        documentation: String,
        parameters: Vec<Parameter>,
        return_type: DataType,
        function: fn(Vec<Value>) -> Value
    ) -> Arc<Self> {
        #[cfg(not(feature = "doc"))]
        let _ = documentation;
        Arc::new_cyclic(|me| Self {
            identifier,
            #[cfg(feature = "doc")]
            documentation,
            parameters,
            return_type,
            function,
            auto_reference: me.clone(),
        })
    }
}

impl Function for CoreFunction {

    fn return_type(&self) -> &DataType {
        &self.return_type
    }

    fn function(&self) -> fn(Vec<Value>) -> Value {
        self.function
    }
}

impl Identified for CoreFunction {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl Documented for CoreFunction {
    fn documentation(&self) -> &str {
        #[cfg(feature = "doc")]
        {&self.documentation}
        #[cfg(not(feature = "doc"))]
        {&""}
    }
}

impl OrderedParameterized for CoreFunction {

    fn parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    fn as_ordered_parameterized(&self) -> Arc<dyn OrderedParameterized> {
        self.auto_reference.upgrade().unwrap()
    }
}

impl Display for CoreFunction {
    
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "function {}({})",
            self.identifier.to_string(),
            self.parameters().iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "),
        )?;

        Ok(())
        
    }
}

