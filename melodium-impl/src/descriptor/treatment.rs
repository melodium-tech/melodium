
use core::fmt::*;
use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock, Mutex};
use melodium_common::descriptor::{Treatment as TreatmentDescriptor, Identified, Identifier, Model, Parameter, Parameterized, Output, Input, Context, Buildable, TreatmentBuildMode, Documented};
use crate::design::treatment::Treatment as Designer;

#[derive(Debug)]
pub struct Treatment {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    models: HashMap<String, Arc<dyn Model>>,
    parameters: HashMap<String, Parameter>,
    inputs: HashMap<String, Input>,
    outputs: HashMap<String, Output>,
    contexts: HashMap<String, Arc<Context>>,
    designer: Mutex<Option<Arc<RwLock<Designer>>>>,
    auto_reference: Weak<Self>,
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
        let mut option_designer = self.designer.lock().expect("Mutex poisoned");

        if let Some(designer_ref) = &*option_designer {
            TreatmentBuildMode::Designed(designer_ref.clone())
        }
        else {
            let new_designer = Arc::new(RwLock::new(Designer{}));

            *option_designer = Some(new_designer.clone());

            TreatmentBuildMode::Designed(new_designer)
        }
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
        &self.contexts
    }

    fn source_from(&self) -> &HashMap<Arc<dyn Model>, Vec<String>> {
        lazy_static!(
            static ref HASHMAP: HashMap<Arc<dyn Model>, Vec<String>> = HashMap::new();
        );
        &HASHMAP
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
