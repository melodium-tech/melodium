use core::fmt::{Display, Formatter, Result};
use melodium_common::descriptor::{
    Attribuable, Attributes, Context as ContextDescriptor, DataType, DescribedType, Documented,
    Identified, Identifier,
};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::Arc;

#[derive(Debug)]
pub struct Context {
    identifier: Identifier,
    values: HashMap<String, DataType>,
    #[cfg(feature = "doc")]
    documentation: String,
    attributes: Attributes,
}

impl Context {
    pub fn new(
        identifier: Identifier,
        values: Vec<(&str, DataType)>,
        #[cfg(feature = "doc")] documentation: String,
        #[cfg(not(feature = "doc"))] _documentation: String,
        attributes: Attributes,
    ) -> Arc<Self> {
        Arc::new(Self {
            identifier,
            values: HashMap::from_iter(values.into_iter().map(|v| (v.0.to_string(), v.1))),
            #[cfg(feature = "doc")]
            documentation,
            attributes,
        })
    }
}

impl Attribuable for Context {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl ContextDescriptor for Context {
    fn name(&self) -> &str {
        &self.identifier.name()
    }

    fn values(&self) -> &HashMap<String, DataType> {
        &self.values
    }
}

impl Documented for Context {
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

impl Identified for Context {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    fn make_use(&self, identifier: &Identifier) -> bool {
        self.values.values().any(|val| {
            DescribedType::from(val)
                .final_type()
                .data()
                .map(|data| data.identifier() == identifier || data.make_use(identifier))
                .unwrap_or(false)
        })
    }

    fn uses(&self) -> Vec<Identifier> {
        self.values
            .values()
            .filter_map(|val| {
                DescribedType::from(val).final_type().data().map(|data| {
                    let mut uses = vec![data.identifier().clone()];
                    uses.extend(data.uses());
                    uses
                })
            })
            .flatten()
            .collect()
    }
}

impl Display for Context {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "context {}", self.identifier.to_string())?;

        for (name, dt) in &self.values {
            write!(f, "{}: {}", name, dt)?
        }

        Ok(())
    }
}
