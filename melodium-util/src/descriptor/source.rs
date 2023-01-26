use core::fmt::{Display, Formatter, Result};
use melodium_common::descriptor::{
    Buildable, Context, Documented, Identified, Identifier, Input, Model, Output, Parameter,
    Parameterized, Treatment as TreatmentDescriptor, TreatmentBuildMode,
};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct Source {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    models: HashMap<String, Arc<dyn Model>>,
    parameters: HashMap<String, Parameter>,
    outputs: HashMap<String, Output>,
    source_from: HashMap<Arc<dyn Model>, Vec<String>>,
    auto_reference: Weak<Self>,
}

impl Source {
    pub fn new(
        identifier: Identifier,
        documentation: String,
        models: Vec<(String, Arc<dyn Model>)>,
        source_from: HashMap<Arc<dyn Model>, Vec<String>>,
        parameters: Vec<Parameter>,
        outputs: Vec<Output>,
    ) -> Arc<Self> {
        #[cfg(not(feature = "doc"))]
        let _ = documentation;
        Arc::new_cyclic(|me| Self {
            identifier,
            #[cfg(feature = "doc")]
            documentation,
            models: HashMap::from_iter(models.iter().map(|m| (m.0.to_string(), Arc::clone(&m.1)))),
            parameters: HashMap::from_iter(
                parameters.iter().map(|p| (p.name().to_string(), p.clone())),
            ),
            outputs: HashMap::from_iter(outputs.iter().map(|o| (o.name().to_string(), o.clone()))),
            source_from,
            auto_reference: me.clone(),
        })
    }
}

impl Identified for Source {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl Documented for Source {
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

impl Parameterized for Source {
    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }
}

impl Buildable<TreatmentBuildMode> for Source {
    fn build_mode(&self) -> TreatmentBuildMode {
        TreatmentBuildMode::Source(self.auto_reference.clone())
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "treatment {}", self.identifier.to_string())?;

        if !self.models.is_empty() {
            write!(
                f,
                "[{}]",
                self.models
                    .iter()
                    .map(|(n, m)| format!("{}: {}", n, m.identifier().to_string()))
                    .collect::<Vec<_>>()
                    .join(", "),
            )?;
        }

        write!(
            f,
            "({})",
            self.parameters()
                .iter()
                .map(|(_, p)| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )?;

        Ok(())
    }
}

impl TreatmentDescriptor for Source {
    fn inputs(&self) -> &HashMap<String, Input> {
        lazy_static! {
            static ref HASHMAP: HashMap<String, Input> = HashMap::new();
        };
        &HASHMAP
    }

    fn outputs(&self) -> &HashMap<String, Output> {
        &self.outputs
    }

    fn models(&self) -> &HashMap<String, Arc<dyn Model>> {
        &self.models
    }

    fn contexts(&self) -> &HashMap<String, Arc<Context>> {
        lazy_static! {
            static ref HASHMAP: HashMap<String, Arc<Context>> = HashMap::new();
        };
        &HASHMAP
    }

    fn source_from(&self) -> &HashMap<Arc<dyn Model>, Vec<String>> {
        &self.source_from
    }

    fn as_identified(&self) -> Arc<dyn Identified> {
        self.auto_reference.upgrade().unwrap()
    }

    fn as_buildable(&self) -> Arc<dyn Buildable<TreatmentBuildMode>> {
        self.auto_reference.upgrade().unwrap()
    }

    fn as_parameterized(&self) -> Arc<dyn Parameterized> {
        self.auto_reference.upgrade().unwrap()
    }
}
