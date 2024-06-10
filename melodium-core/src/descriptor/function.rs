use core::fmt::{Display, Formatter, Result};
use melodium_common::descriptor::{
    Attribuable, Attributes, DataType, DescribedType, Documented, Function as FunctionDescriptor,
    Generic, Generics, Identified, Identifier, OrderedParameterized, Parameter,
};
use melodium_common::executive::Value;
use std::collections::HashMap;
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct Function {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    attributes: Attributes,
    generics: Vec<Generic>,
    parameters: Vec<Parameter>,
    return_type: DescribedType,
    function: fn(HashMap<String, DataType>, Vec<Value>) -> Value,
    auto_reference: Weak<Self>,
}

impl Function {
    pub fn new(
        identifier: Identifier,
        documentation: String,
        attributes: Attributes,
        generics: Vec<Generic>,
        parameters: Vec<Parameter>,
        return_type: DescribedType,
        function: fn(HashMap<String, DataType>, Vec<Value>) -> Value,
    ) -> Arc<Self> {
        #[cfg(not(feature = "doc"))]
        let _ = documentation;
        Arc::new_cyclic(|me| Self {
            identifier,
            #[cfg(feature = "doc")]
            documentation,
            attributes,
            generics,
            parameters,
            return_type,
            function,
            auto_reference: me.clone(),
        })
    }
}

impl Attribuable for Function {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl FunctionDescriptor for Function {
    fn return_type(&self) -> &DescribedType {
        &self.return_type
    }

    fn function(&self) -> fn(HashMap<String, DataType>, Vec<Value>) -> Value {
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

    fn make_use(&self, identifier: &Identifier) -> bool {
        self.return_type
            .final_type()
            .data()
            .map(|data| data.identifier() == identifier || data.make_use(identifier))
            .unwrap_or(false)
            || self.parameters.iter().any(|param| {
                param
                    .described_type()
                    .final_type()
                    .data()
                    .map(|data| data.identifier() == identifier || data.make_use(identifier))
                    .unwrap_or(false)
            })
    }

    fn uses(&self) -> Vec<Identifier> {
        let mut uses = Vec::new();
        if let Some(data) = self.return_type.final_type().data() {
            uses.push(data.identifier().clone());
            uses.extend(data.uses());
        }
        for param in &self.parameters {
            if let Some(data) = param.described_type().final_type().data() {
                uses.push(data.identifier().clone());
                uses.extend(data.uses());
            }
        }
        uses
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

impl Generics for Function {
    fn generics(&self) -> &Vec<Generic> {
        &self.generics
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
