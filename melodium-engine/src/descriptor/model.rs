use crate::design::Model as Design;
use crate::designer::{Model as Designer, Reference};
use crate::error::{LogicError, LogicResult};
use core::fmt::{Display, Formatter, Result as FmtResult};
use melodium_common::descriptor::{
    Buildable, Collection, Context, Documented, Identified, Identifier, Model as ModelDescriptor,
    ModelBuildMode, Parameter, Parameterized, Status,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock, Weak};

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
    pub fn new(identifier: Identifier, base_model: &Arc<dyn ModelDescriptor>) -> Self {
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

    pub fn designer(
        &self,
        collection: Arc<Collection>,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<Arc<RwLock<Designer>>> {
        if self.auto_reference.strong_count() == 0 {
            return Status::new_failure(
                LogicError::uncommited_descriptor(2, self.identifier.clone(), design_reference)
                    .into(),
            );
        }

        let mut option_designer = self.designer.lock().expect("Mutex poisoned");

        if let Some(designer_ref) = &*option_designer {
            Status::new_success(designer_ref.clone())
        } else {
            let new_designer = Designer::new(
                &self.auto_reference.upgrade().unwrap(),
                collection,
                design_reference,
            );

            *option_designer = Some(new_designer.clone());

            Status::new_success(new_designer)
        }
    }

    pub fn commit_design(&self) -> LogicResult<()> {
        let option_designer = self.designer.lock().expect("Mutex poisoned");
        let mut option_design = self.design.lock().expect("Mutex poisoned");

        if let Some(designer_ref) = &*option_designer {
            let designer = designer_ref.read().unwrap();
            let mut result_design = designer.design();

            if result_design.is_success() && result_design.has_errors() {
                result_design =
                    result_design.and(Status::new_failure(LogicError::erroneous_design(
                        66,
                        self.identifier.clone(),
                        designer.design_reference().clone(),
                    )));
            }

            result_design.and_then(|design| {
                *option_design = Some(Arc::new(design));
                Status::new_success(())
            })
        } else {
            Status::new_failure(LogicError::no_designer(65, self.identifier.clone(), None))
        }
    }

    pub fn design(&self) -> LogicResult<Arc<Design>> {
        let option_design = self.design.lock().expect("Mutex poisoned");

        option_design
            .as_ref()
            .map(|design| Arc::clone(design))
            .ok_or_else(|| LogicError::unavailable_design(4, self.identifier.clone(), None).into())
            .into()
    }

    pub fn set_documentation(&mut self, documentation: &str) {
        #[cfg(feature = "doc")]
        {
            self.documentation = String::from(documentation);
        }
        #[cfg(not(feature = "doc"))]
        let _ = documentation;
    }

    pub fn add_parameter(&mut self, parameter: Parameter) {
        self.parameters
            .insert(parameter.name().to_string(), parameter);
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
            auto_reference: me.clone(),
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
        {
            &self.documentation
        }
        #[cfg(not(feature = "doc"))]
        {
            &""
        }
    }
}

impl Parameterized for Model {
    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }

    fn as_identified(&self) -> Arc<dyn Identified> {
        self.auto_reference.upgrade().unwrap()
    }
}

impl Buildable<ModelBuildMode> for Model {
    fn build_mode(&self) -> ModelBuildMode {
        ModelBuildMode::Designed()
    }

    fn make_use(&self, identifier: &Identifier) -> bool {
        self.base_model.identifier() == identifier
    }
}

impl ModelDescriptor for Model {
    fn is_core_model(&self) -> bool {
        false
    }

    fn base_model(&self) -> Option<Arc<dyn ModelDescriptor>> {
        Some(Arc::clone(&self.base_model))
    }

    fn sources(&self) -> &HashMap<String, Vec<Arc<dyn Context>>> {
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
        write!(
            f,
            "model {}({})",
            self.identifier.to_string(),
            self.parameters()
                .iter()
                .map(|(_, p)| p.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        )?;

        Ok(())
    }
}
