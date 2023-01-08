
use core::fmt::{Display, Formatter, Result as FmtResult};
use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock, Mutex};
use melodium_common::descriptor::{Identified, Identifier, Model as ModelDescriptor, Parameter, Parameterized, Context, Buildable, ModelBuildMode, Documented};
use crate::designer::Model as Designer;
use crate::design::Model as Design;
use crate::error::LogicError;

#[derive(Debug)]
pub struct Model {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    base_model: Arc<dyn ModelDescriptor>,
    parameters: HashMap<String, Parameter>,
    designer: Mutex<Option<Arc<RwLock<Designer>>>>,
    design: Mutex<Option<Arc<Design>>>,
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
            design: Mutex::new(None),
            auto_reference: Weak::default(),
        }
    }

    pub fn reset_designer(&self) {
        let mut option_designer = self.designer.lock().expect("Mutex poisoned");
        *option_designer = None;
    }

    pub fn designer(&self) -> Result<Arc<RwLock<Designer>>, LogicError> {

        if self.auto_reference.strong_count() == 0 {
            return Err(LogicError::uncommited_descriptor())
        }

        let mut option_designer = self.designer.lock().expect("Mutex poisoned");

        if let Some(designer_ref) = &*option_designer {
            Ok(designer_ref.clone())
        }
        else {
            let new_designer = Designer::new(&self.auto_reference.upgrade().unwrap());

            *option_designer = Some(new_designer.clone());

            Ok(new_designer)
        }
    }

    pub fn commit_design(&self) -> Result<(), LogicError> {

        let mut option_designer = self.designer.lock().expect("Mutex poisoned");
        let mut option_design = self.design.lock().expect("Mutex poisoned");

        if let Some(designer_ref) = &*option_designer {

            let designer = designer_ref.read().unwrap();
            *option_design = Some(Arc::new(designer.design()?));
        }

        Ok(())
    }

    pub fn design(&self) -> Result<Arc<Design>, LogicError> {

        let mut option_design = self.design.lock().expect("Mutex poisoned");

        option_design.as_ref().map(|design| Arc::clone(design)).ok_or_else(|| LogicError::unavailable_design())
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
            design: self.design,
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
        ModelBuildMode::Designed()
    }
}

impl ModelDescriptor for Model {

    fn is_core_model(&self) -> bool {
        false
    }

    fn base_model(&self) -> Option<Arc<dyn ModelDescriptor>> {
        Some(Arc::clone(&self.base_model))
    }

    fn sources(&self) -> &HashMap<String, Vec<Arc<Context>>> {
        self.base_model.sources()
    }

    fn as_identified(&self) -> Arc<dyn Identified> {
        self.auto_reference.upgrade().unwrap()
    }

    fn as_buildable(&self) -> Arc<dyn Buildable<ModelBuildMode>> {
        self.auto_reference.upgrade().unwrap()
    }

    fn as_parameterized(&self) -> Arc<dyn Parameterized> {
        self.auto_reference.upgrade().unwrap()
    }
}

impl Display for Model {
    
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "model {}({})",
            self.identifier.to_string(),
            self.parameters().iter().map(|(_, p)| p.to_string()).collect::<Vec<_>>().join(", "),
        )?;

        Ok(())
    }
}


