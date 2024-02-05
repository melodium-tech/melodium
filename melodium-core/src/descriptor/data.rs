use core::fmt::{Display, Formatter};
use melodium_common::descriptor::{
    Attribuable, Attributes, Data as DataDescriptor, DataTrait, Documented, Identified, Identifier,
};
use std::sync::Arc;

#[derive(Debug)]
pub struct Data {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    attributes: Attributes,
    implements: Vec<DataTrait>,
}

impl Data {
    pub fn new(
        identifier: Identifier,
        documentation: String,
        attributes: Attributes,
        implements: Vec<DataTrait>,
    ) -> Arc<Self> {
        #[cfg(not(feature = "doc"))]
        let _ = documentation;
        Arc::new(Self {
            identifier,
            #[cfg(feature = "doc")]
            documentation,
            attributes,
            implements,
        })
    }
}

impl Attribuable for Data {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl Identified for Data {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl Documented for Data {
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

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "data {}", self.identifier.to_string(),)?;

        Ok(())
    }
}

impl DataDescriptor for Data {
    fn implements(&self) -> &[DataTrait] {
        &self.implements
    }
}
