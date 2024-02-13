//! Module dedicated to InstanciedModel semantic analysis.

use super::assignative_element::{AssignativeElement, AssignativeElementType};
use super::assigned_parameter::AssignedParameter;
use super::common::Node;
use super::common::Reference;
use super::declarative_element::DeclarativeElement;
use super::model::Model;
use super::r#use::Use;
use super::treatment::Treatment;
use crate::error::ScriptError;
use crate::path::Path;
use crate::text::Instanciation as TextInstanciation;
use crate::ScriptResult;
use melodium_common::descriptor::{Collection, Identifier};
use melodium_engine::designer::ModelInstanciation as ModelInstanciationDesigner;
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of a model instanciation.
///
/// It owns the whole [text instanciation](TextInstanciation).
#[derive(Debug)]
pub struct ModelInstanciation {
    pub text: TextInstanciation,

    pub treatment: Weak<RwLock<Treatment>>,

    pub name: String,
    pub r#type: RefersTo,
    pub parameters: Vec<Arc<RwLock<AssignedParameter>>>,

    pub type_identifier: Option<Identifier>,
}

/// Enumeration managing what model instanciation refers to.
///
/// This is a convenience enum, as a model instanciation may refer either on a [Use] or a [Model].
/// The `Unknown` variant is aimed to hold a reference-to-nothing, as long as `make_references() hasn't been called.
#[derive(Debug)]
pub enum RefersTo {
    Unknown(Reference<()>),
    Use(Reference<Use>),
    Model(Reference<Model>),
}

impl ModelInstanciation {
    /// Create a new semantic model instanciation, based on textual instanciation.
    ///
    /// * `treatment`: the parent treatment owning this instanciation.
    /// * `text`: the textual instanciation.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    ///
    pub fn new(
        treatment: Arc<RwLock<Treatment>>,
        text: TextInstanciation,
    ) -> ScriptResult<Arc<RwLock<Self>>> {
        let model = Arc::<RwLock<Self>>::new(RwLock::new(Self {
            text: text.clone(),
            treatment: Arc::downgrade(&treatment),
            name: text.name.string.clone(),
            r#type: RefersTo::Unknown(Reference::new(text.r#type.string)),
            parameters: Vec::new(),
            type_identifier: None,
        }));
        let mut result = ScriptResult::new_success(Arc::clone(&model));

        {
            let borrowed_treatment = treatment.read().unwrap();

            if let Some(_) = borrowed_treatment.find_model_instanciation(&text.name.string) {
                result = result.and_degrade_failure(ScriptResult::new_failure(
                    ScriptError::already_used_name(128, text.name),
                ));
            }
        }

        for p in text.parameters {
            if let Some(assigned_parameter) = result.merge_degrade_failure(AssignedParameter::new(
                Arc::clone(&model) as Arc<RwLock<dyn AssignativeElement>>,
                p,
            )) {
                model.write().unwrap().parameters.push(assigned_parameter);
            }
        }

        result
    }

    pub fn make_design(
        &self,
        designer: &Arc<RwLock<ModelInstanciationDesigner>>,
        collection: &Arc<Collection>,
    ) -> ScriptResult<()> {
        let mut designer = designer.write().unwrap();
        let mut result = ScriptResult::new_success(());
        for rc_assignation in &self.parameters {
            let borrowed_assignation = rc_assignation.read().unwrap();

            if let Some(assignation_designer) =
                result.merge_degrade_failure(ScriptResult::from(designer.add_parameter(
                    &borrowed_assignation.name,
                    Some(borrowed_assignation.text.name.into_ref()),
                )))
            {
                result = result.and_degrade_failure(
                    borrowed_assignation.make_design(&assignation_designer, collection),
                );
            }
        }

        result = result.and_degrade_failure(ScriptResult::from(designer.validate()));

        result
    }
}

impl AssignativeElement for ModelInstanciation {
    fn assignative_element(&self) -> AssignativeElementType {
        AssignativeElementType::ModelInstanciation(self)
    }

    fn associated_declarative_element(&self) -> Arc<RwLock<dyn DeclarativeElement>> {
        self.treatment.upgrade().unwrap() as Arc<RwLock<dyn DeclarativeElement>>
    }

    fn find_assigned_parameter(&self, name: &str) -> Option<&Arc<RwLock<AssignedParameter>>> {
        self.parameters
            .iter()
            .find(|&a| a.read().unwrap().name == name)
    }
}

impl Node for ModelInstanciation {
    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {
        let mut children: Vec<Arc<RwLock<dyn Node>>> = Vec::new();

        self.parameters
            .iter()
            .for_each(|p| children.push(Arc::clone(&p) as Arc<RwLock<dyn Node>>));

        children
    }

    fn make_references(&mut self, path: &Path) -> ScriptResult<()> {
        if let RefersTo::Unknown(reference) = &self.r#type {
            let rc_treatment = self.treatment.upgrade().unwrap();
            let borrowed_treatment = rc_treatment.read().unwrap();
            let rc_script = borrowed_treatment.script.upgrade().unwrap();
            let borrowed_script = rc_script.read().unwrap();

            let r#use = borrowed_script.find_use(&reference.name);
            if r#use.is_some() {
                let r#use = r#use.unwrap();

                self.type_identifier = r#use.read().unwrap().identifier.clone();

                self.r#type = RefersTo::Use(Reference {
                    name: reference.name.clone(),
                    reference: Some(Arc::downgrade(r#use)),
                });
            } else {
                let model = borrowed_script.find_model(&reference.name);
                if model.is_some() {
                    self.type_identifier = path.to_identifier(&reference.name);

                    self.r#type = RefersTo::Model(Reference {
                        name: reference.name.clone(),
                        reference: Some(Arc::downgrade(model.unwrap())),
                    });
                } else {
                    return ScriptResult::new_failure(ScriptError::unimported_element(
                        130,
                        self.text.r#type.clone(),
                    ));
                }
            }
        }

        ScriptResult::new_success(())
    }
}
