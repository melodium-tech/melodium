use core::fmt::{Display, Formatter};
use std::sync::Arc;
use melodium_common::descriptor::{DataTrait, Identifier, Attributes, Object as ObjectDescriptor, Attribuable, Identified, Documented};


#[derive(Debug)]
pub struct Object {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    attributes: Attributes,
    implements: Vec<DataTrait>,
}


impl Object {
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

impl Attribuable for Object {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl Identified for Object {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl Documented for Object {
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

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "object {}",
            self.identifier.to_string(),
        )?;

        Ok(())
    }
}

impl ObjectDescriptor for Object {
    fn implements(&self) -> &[DataTrait] {
        &self.implements
    }
}
