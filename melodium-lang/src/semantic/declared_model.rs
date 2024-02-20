//! Module dedicated to DeclaredModel semantic analysis.

use super::common::Node;
use super::common::Reference;
use super::model::Model;
use super::model_instanciation::ModelInstanciation;
use super::r#use::Use;
use super::treatment::Treatment;
use crate::error::ScriptError;
use crate::path::Path;
use crate::text::word::PositionnedString;
use crate::text::Parameter as TextParameter;
use crate::ScriptResult;
use melodium_common::descriptor::VersionReq;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of a declared model.
///
/// It owns optionnally the whole [text parameter](TextParameter),
/// depending on explicit or implicit declaration.
#[derive(Debug)]
pub struct DeclaredModel {
    pub text: Option<TextParameter>,

    pub treatment: Weak<RwLock<Treatment>>,

    pub name: String,
    pub refers: RefersTo,
}

/// Enumeration managing what declared model type refers to.
///
/// This is a convenience enum, as a declared model type may refer either on a [Use], a [Model], or an [ModelInstanciation].
/// The `Unknown` variant is aimed to hold a reference-to-nothing, as long as `make_references() hasn't been called.
#[derive(Debug)]
pub enum RefersTo {
    Unknown(Reference<()>),
    Use(Reference<Use>),
    Model(Reference<Model>),
    InstanciedModel(Reference<ModelInstanciation>),
}

impl DeclaredModel {
    /// Create a new semantic declaration of model, from an instancied model.
    ///
    /// When using this creation method, the `text` member will be `None`.
    ///
    /// * `instancied_model`: the InstanciedModel to use as declaration.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    pub fn from_instancied_model(
        instancied_model: Arc<RwLock<ModelInstanciation>>,
    ) -> ScriptResult<Arc<RwLock<Self>>> {
        let borrowed_instancied_model = instancied_model.read().unwrap();

        let treatment = borrowed_instancied_model.treatment.upgrade().unwrap();
        let name = borrowed_instancied_model.name.clone();

        Self::make(treatment, borrowed_instancied_model.text.name.clone()).and_then(
            |declared_model| {
                declared_model.write().unwrap().refers = RefersTo::InstanciedModel(Reference {
                    name: name,
                    reference: Some(Arc::downgrade(&instancied_model)),
                });
                ScriptResult::new_success(declared_model)
            },
        )
    }

    /// Create a new semantic declaration of model, based on textual parameter.
    ///
    /// * `treatment`: the treatment owning this declaration.
    /// * `text`: the textual model.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    ///
    pub fn new(
        treatment: Arc<RwLock<Treatment>>,
        text: TextParameter,
    ) -> ScriptResult<Arc<RwLock<Self>>> {
        let mut result = ScriptResult::new_success(());

        let refers_string;
        if let Some(r#type) = &text.r#type {
            if !r#type.level_structure.is_empty() {
                result = result.and_degrade_failure(ScriptResult::new_failure(
                    ScriptError::structure_forbidden(138, text.name.clone()),
                ));
            }

            refers_string = r#type.name.string.clone();
        } else {
            refers_string = String::new();
            result = result.and_degrade_failure(ScriptResult::new_failure(
                ScriptError::missing_type(139, text.name.clone()),
            ));
        }

        if text.value.is_some() {
            result = result.and_degrade_failure(ScriptResult::new_failure(
                ScriptError::default_forbidden(140, text.name.clone()),
            ));
        }

        result.and_then(|_| {
            Self::make(treatment, text.name.clone()).and_then(|declared_model| {
                let mut borrowed_declared_model = declared_model.write().unwrap();
                borrowed_declared_model.text = Some(text);
                borrowed_declared_model.refers = RefersTo::Unknown(Reference::new(refers_string));
                ScriptResult::new_success(declared_model.clone())
            })
        })
    }

    fn make(
        treatment: Arc<RwLock<Treatment>>,
        name: PositionnedString,
    ) -> ScriptResult<Arc<RwLock<Self>>> {
        let borrowed_treatment = treatment.read().unwrap();

        let declared_model = borrowed_treatment.find_declared_model(&name.string.clone());
        if declared_model.is_some() {
            ScriptResult::new_failure(ScriptError::already_declared(141, name))
        } else {
            ScriptResult::new_success(Arc::<RwLock<Self>>::new(RwLock::new(Self {
                treatment: Arc::downgrade(&treatment),
                name: name.string.clone(),
                text: None,
                refers: RefersTo::Unknown(Reference::new(name.string)),
            })))
        }
    }

    pub fn comes_from_instancied(&self) -> bool {
        match self.refers {
            RefersTo::InstanciedModel(_) => true,
            _ => false,
        }
    }
}

impl Node for DeclaredModel {
    fn make_references(
        &mut self,
        _path: &Path,
        _versions: &HashMap<String, VersionReq>,
    ) -> ScriptResult<()> {
        // Reference to an instancied model already been done through Self::from_instancied_model
        // so we only look for reference to a use.
        if let RefersTo::Unknown(reference) = &self.refers {
            let rc_treatment = self.treatment.upgrade().unwrap();
            let borrowed_treatment = rc_treatment.read().unwrap();
            let rc_script = borrowed_treatment.script.upgrade().unwrap();
            let borrowed_script = rc_script.read().unwrap();

            if let Some(model) = borrowed_script.find_model(&reference.name) {
                self.refers = RefersTo::Model(Reference {
                    name: reference.name.clone(),
                    reference: Some(Arc::downgrade(model)),
                });
            } else if let Some(r#use) = borrowed_script.find_use(&reference.name) {
                self.refers = RefersTo::Use(Reference {
                    name: reference.name.clone(),
                    reference: Some(Arc::downgrade(r#use)),
                });
            } else {
                return ScriptResult::new_failure(ScriptError::unimported_element(
                    142,
                    self.text.as_ref().unwrap().name.clone(),
                ));
            }
        }

        ScriptResult::new_success(())
    }
}
