
use core::fmt::*;
use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock, Mutex};
use melodium_common::descriptor::{Identified, Identifier, Model as ModelDescriptor, Parameter, Parameterized, Context, Buildable, ModelBuildMode, Documented};
use crate::design::model::Model as Designer;

#[derive(Debug)]
pub struct Model {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    base_model: Arc<dyn ModelDescriptor>,
    parameters: HashMap<String, Parameter>,
    designer: Mutex<Option<Arc<RwLock<Designer>>>>,
    auto_reference: Weak<Self>,
}

impl Model {
    pub fn new(
        identifier: Identifier,
        base_model: &Arc<dyn ModelDescriptor>
    ) -> Self {
        Self {
            identifier,
            #[cfg(feature = "doc")]
            documentation: String::new(),
            base_model: Arc::clone(base_model),
            parameters: HashMap::new(),
            designer: Mutex::new(None),
            auto_reference: Weak::default(),
        }
    }

    pub fn set_documentation(&mut self, documentation: &str) {
        #[cfg(feature = "doc")]
        {self.documentation = String::from(documentation);}
        #[cfg(not(feature = "doc"))]
        let _ = documentation;
    }

    pub fn add_parameter(&mut self, parameter: Parameter) {
        self.parameters.insert(parameter.name().to_string(), parameter);
    }

    pub fn commit(self) -> Arc<Self> {
        Arc::new_cyclic(|me| Self {
            identifier: self.identifier,
            #[cfg(feature = "doc")]
            documentation: self.documentation,
            base_model: self.base_model,
            parameters: self.parameters,
            designer: self.designer,
            auto_reference: me.clone()
        })
    }
}

impl Identified for Model {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl Documented for Model {
    fn documentation(&self) -> &str {
        #[cfg(feature = "doc")]
        {&self.documentation}
        #[cfg(not(feature = "doc"))]
        {&""}
    }
}

impl Parameterized for Model {

    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }
}

impl Buildable<ModelBuildMode> for Model {
    fn build_mode(&self) -> ModelBuildMode {
        let mut option_designer = self.designer.lock().expect("Mutex poisoned");

        if let Some(designer_ref) = &*option_designer {
            ModelBuildMode::Designed(designer_ref.clone())
        }
        else {
            let new_designer = Arc::new(RwLock::new(Designer{}));

            *option_designer = Some(new_designer.clone());

            ModelBuildMode::Designed(new_designer)
        }
    }
}

impl ModelDescriptor for Model {

    fn is_core_model(&self) -> bool {
        false
    }

    fn base_model(&self) -> Arc<dyn ModelDescriptor> {
        Arc::clone(&self.base_model)
    }

    fn sources(&self) -> &HashMap<String, Vec<Arc<Context>>> {
        self.base_model.sources()
    }

    fn as_identified(&self) -> Arc<dyn Identified> {
        self.auto_reference.upgrade().unwrap()
    }

    fn as_parameterized(&self) -> Arc<dyn Parameterized> {
        self.auto_reference.upgrade().unwrap()
    }
}

impl Display for Model {
    
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "model {}({})",
            self.identifier.to_string(),
            self.parameters().iter().map(|(_, p)| p.to_string()).collect::<Vec<_>>().join(", "),
        )?;

        Ok(())
    }
}


