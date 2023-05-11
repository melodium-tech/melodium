use core::fmt::{Display, Formatter, Result};
use melodium_common::descriptor::{
    Buildable, Context, Documented, Identified, Identifier, Input, Model, Output, Parameter,
    Parameterized, Treatment as TreatmentDescriptor, TreatmentBuildMode,
};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct Source {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    models: HashMap<String, Arc<dyn Model>>,
    outputs: HashMap<String, Output>,
    source_from: HashMap<String, Vec<String>>,
    auto_reference: Weak<Self>,
}

impl Source {
    pub fn new(
        identifier: Identifier,
        documentation: String,
        models: Vec<(String, Arc<dyn Model>)>,
        source_from: Vec<(String, Vec<String>)>,
        outputs: Vec<Output>,
    ) -> Arc<Self> {
        #[cfg(not(feature = "doc"))]
        let _ = documentation;
        Arc::new_cyclic(|me| Self {
            identifier,
            #[cfg(feature = "doc")]
            documentation,
            models: HashMap::from_iter(models.into_iter().map(|(n, m)| (n.to_string(), m))),
            outputs: HashMap::from_iter(outputs.into_iter().map(|o| (o.name().to_string(), o))),
            source_from: HashMap::from_iter(source_from.into_iter()),
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
        static HASHMAP: OnceCell<HashMap<String, Parameter>> = OnceCell::new();
        HASHMAP.get_or_init(|| HashMap::new())
    }

    fn as_identified(&self) -> Arc<dyn Identified> {
        self.auto_reference.upgrade().unwrap()
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
        static HASHMAP: OnceCell<HashMap<String, Input>> = OnceCell::new();
        HASHMAP.get_or_init(|| HashMap::new())
    }

    fn outputs(&self) -> &HashMap<String, Output> {
        &self.outputs
    }

    fn models(&self) -> &HashMap<String, Arc<dyn Model>> {
        &self.models
    }

    fn contexts(&self) -> &HashMap<String, Arc<dyn Context>> {
        static HASHMAP: OnceCell<HashMap<String, Arc<dyn Context>>> = OnceCell::new();
        HASHMAP.get_or_init(|| HashMap::new())
    }

    fn source_from(&self) -> &HashMap<String, Vec<String>> {
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
