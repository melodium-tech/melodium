use core::fmt::{Display, Formatter, Result};
use melodium_common::descriptor::{
    Buildable, Context, Documented, Identified, Identifier, Input, Model, Output, Parameter,
    Parameterized, Treatment as TreatmentDescriptor, TreatmentBuildMode,
};
use melodium_common::executive::Treatment as ExecutiveTreatment;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::{Arc, Weak};
use once_cell::sync::OnceCell;

#[derive(Debug)]
pub struct Treatment {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    models: HashMap<String, Arc<dyn Model>>,
    parameters: HashMap<String, Parameter>,
    inputs: HashMap<String, Input>,
    outputs: HashMap<String, Output>,
    source_from: HashMap<String, Vec<String>>,
    build_fn: fn() -> Arc<dyn ExecutiveTreatment>,
    auto_reference: Weak<Self>,
}

impl Treatment {
    pub fn new(
        identifier: Identifier,
        documentation: String,
        models: Vec<(String, Arc<dyn Model>)>,
        source_from: Vec<(String, Vec<String>)>,
        parameters: Vec<Parameter>,
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        build_fn: fn() -> Arc<dyn ExecutiveTreatment>,
    ) -> Arc<Self> {
        #[cfg(not(feature = "doc"))]
        let _ = documentation;
        Arc::new_cyclic(|me| Self {
            identifier,
            #[cfg(feature = "doc")]
            documentation,
            models: HashMap::from_iter(models.into_iter().map(|(n, m)| (n, m))),
            parameters: HashMap::from_iter(
                parameters.into_iter().map(|p| (p.name().to_string(), p)),
            ),
            inputs: HashMap::from_iter(inputs.into_iter().map(|i| (i.name().to_string(), i))),
            outputs: HashMap::from_iter(outputs.into_iter().map(|o| (o.name().to_string(), o))),
            source_from: HashMap::from_iter(source_from.into_iter()),
            build_fn,
            auto_reference: me.clone(),
        })
    }
}

impl Identified for Treatment {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl Documented for Treatment {
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

impl Parameterized for Treatment {
    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }
}

impl Buildable<TreatmentBuildMode> for Treatment {
    fn build_mode(&self) -> TreatmentBuildMode {
        TreatmentBuildMode::Compiled(self.build_fn, self.auto_reference.clone())
    }
}

impl Display for Treatment {
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

impl TreatmentDescriptor for Treatment {
    fn inputs(&self) -> &HashMap<String, Input> {
        &self.inputs
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
