use core::fmt::{Display, Formatter, Result};
use melodium_common::descriptor::{
    DataType, Documented, Function as FunctionDescriptor, Identified, Identifier,
    OrderedParameterized, Parameter,
};
use melodium_common::executive::Value;
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct Function {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    parameters: Vec<Parameter>,
    return_type: DataType,
    function: fn(Vec<Value>) -> Value,
    auto_reference: Weak<Self>,
}

impl Function {
    pub fn new(
        identifier: Identifier,
        documentation: String,
        parameters: Vec<Parameter>,
        return_type: DataType,
        function: fn(Vec<Value>) -> Value,
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

impl FunctionDescriptor for Function {
    fn return_type(&self) -> &DataType {
        &self.return_type
    }

    fn function(&self) -> fn(Vec<Value>) -> Value {
        self.function
    }

    fn as_identified(&self) -> Arc<dyn Identified> {
        self.auto_reference.upgrade().unwrap()
    }

    fn as_ordered_parameterized(&self) -> Arc<dyn OrderedParameterized> {
        self.auto_reference.upgrade().unwrap()
    }
}

impl Identified for Function {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl Documented for Function {
    fn documentation(&self) -> &str {
        #[cfg(feature = "doc")]
        {
            &self.documentation
        }
        #[cfg(not(feature = "doc"))]
        {
            &""
        }
    }
}

impl OrderedParameterized for Function {
    fn parameters(&self) -> &Vec<Parameter> {
        &self.parameters
    }

    fn as_identified(&self) -> Arc<dyn Identified> {
        self.auto_reference.upgrade().unwrap()
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "function {}({})",
            self.identifier.to_string(),
            self.parameters()
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        )?;

        Ok(())
    }
}
