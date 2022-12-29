
use core::fmt::{Display, Formatter, Result};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::{Arc, Weak};
use melodium_common::descriptor::{Buildable, Context, Documented, Parameterized, Identified, Identifier, Model, Parameter, Input, Output, Treatment as TreatmentDescriptor, TreatmentBuildMode};
use melodium_common::executive::Treatment as ExecutiveTreatment;

#[derive(Debug)]
pub struct Treatment {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    models: HashMap<String, Arc<dyn Model>>,
    parameters: HashMap<String, Parameter>,
    inputs: HashMap<String, Input>,
    outputs: HashMap<String, Output>,
    source_from: HashMap<Arc<dyn Model>, Vec<String>>,
    build_fn: fn() -> Arc<dyn ExecutiveTreatment>,
    auto_reference: Weak<Self>,
}


impl Treatment {
    pub fn new(
        identifier: Identifier,
        documentation: String,
        models: Vec<(String, Arc<dyn Model>)>,
        source_from: HashMap<Arc<dyn Model>, Vec<String>>,
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
            models: HashMap::from_iter(models.iter().map(|m| (m.0.to_string(), Arc::clone(&m.1)))),
            parameters: HashMap::from_iter(parameters.iter().map(|p| (p.name().to_string(), p.clone()))),
            inputs: HashMap::from_iter(inputs.iter().map(|i| (i.name().to_string(), i.clone()))),
            outputs: HashMap::from_iter(outputs.iter().map(|o| (o.name().to_string(), o.clone()))),
            source_from,
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
        {&self.documentation}
        #[cfg(not(feature = "doc"))]
        {&""}
    }
}

impl Parameterized for Treatment {
    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }
}

impl Buildable<TreatmentBuildMode> for Treatment {
    fn build_mode(&self) -> TreatmentBuildMode {
        TreatmentBuildMode::Compiled(self.build_fn)
    }
}

impl Display for Treatment {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {

        write!(f, "treatment {}", self.identifier.to_string())?;

        if !self.models.is_empty() {
            write!(f, "[{}]",
                self.models.iter().map(|(n, m)| format!("{}: {}", n, m.identifier().to_string())).collect::<Vec<_>>().join(", "),
            )?;
        }

        write!(f, "({})", self.parameters().iter().map(|(_, p)| p.to_string()).collect::<Vec<_>>().join(", "))?;
        
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

    fn contexts(&self) -> &HashMap<String, Arc<Context>> {
        lazy_static!(
            static ref HASHMAP: HashMap<String, Arc<Context>> = HashMap::new();
        );
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
