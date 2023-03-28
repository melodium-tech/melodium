//! Module dedicated to treatment instanciation semantic analysis.

use super::assignative_element::{AssignativeElement, AssignativeElementType};
use super::assigned_model::AssignedModel;
use super::assigned_parameter::AssignedParameter;
use super::common::Node;
use super::common::Reference;
use super::declarative_element::DeclarativeElement;
use super::r#use::Use;
use super::treatment::Treatment;
use crate::error::{wrap_logic_error, ScriptError};
use crate::path::Path;
use crate::text::Instanciation as TextTreatment;
use melodium_common::descriptor::Identifier;
use melodium_engine::designer::TreatmentInstanciation as TreatmentInstanciationDesigner;
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of a treatment.
///
/// It owns the whole [text instanciation](crate::text::Instanciation).
#[derive(Debug)]
pub struct TreatmentInstanciation {
    pub text: TextTreatment,

    pub treatment: Weak<RwLock<Treatment>>,

    pub name: String,
    pub r#type: RefersTo,
    pub models: Vec<Arc<RwLock<AssignedModel>>>,
    pub parameters: Vec<Arc<RwLock<AssignedParameter>>>,

    pub type_identifier: Option<Identifier>,
}

/// Enumeration managing what treatment type refers to.
///
/// This is a convenience enum, as a treatment type may refer either on a [Use](Use) or a [Treatment](Treatment).
/// The `Unknown` variant is aimed to hold a reference-to-nothing, as long as `make_references() hasn't been called.
#[derive(Debug)]
pub enum RefersTo {
    Unknown(Reference<()>),
    Use(Reference<Use>),
    Treatment(Reference<Treatment>),
}

impl TreatmentInstanciation {
    /// Create a new semantic treatment instanciation, based on textual treatment instanciation.
    ///
    /// * `treatment`: the parent treatment that owns this treatment instanciation.
    /// * `text`: the textual treatment.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    ///
    pub fn new(
        treatment: Arc<RwLock<Treatment>>,
        text: TextTreatment,
    ) -> Result<Arc<RwLock<Self>>, ScriptError> {
        let treatment_instanciation = Arc::<RwLock<Self>>::new(RwLock::new(Self {
            text: text.clone(),
            treatment: Arc::downgrade(&treatment),
            name: text.name.string.clone(),
            r#type: RefersTo::Unknown(Reference::new(text.r#type.string)),
            models: Vec::new(),
            parameters: Vec::new(),
            type_identifier: None,
        }));

        {
            let borrowed_treatment = treatment.read().unwrap();

            if let Some(_) = borrowed_treatment.find_treatment_instanciation(&text.name.string) {
                return Err(ScriptError::semantic(
                    "Treatment '".to_string() + &text.name.string + "' is already declared.",
                    text.name.position,
                ));
            }
        }

        for m in text.configuration {
            let assigned_model = AssignedModel::new(
                Arc::clone(&treatment_instanciation) as Arc<RwLock<dyn AssignativeElement>>,
                m,
            )?;
            treatment_instanciation
                .write()
                .unwrap()
                .models
                .push(assigned_model);
        }

        for p in text.parameters {
            let assigned_parameter = AssignedParameter::new(
                Arc::clone(&treatment_instanciation) as Arc<RwLock<dyn AssignativeElement>>,
                p,
            )?;
            treatment_instanciation
                .write()
                .unwrap()
                .parameters
                .push(assigned_parameter);
        }

        Ok(treatment_instanciation)
    }

    pub fn make_design(
        &self,
        designer: &Arc<RwLock<TreatmentInstanciationDesigner>>,
    ) -> Result<(), ScriptError> {
        let mut designer = designer.write().unwrap();

        for rc_model_assignation in &self.models {
            let borrowed_model_assignation = rc_model_assignation.read().unwrap();

            designer
                .add_model(
                    &borrowed_model_assignation.name,
                    &borrowed_model_assignation.model.name,
                )
                .unwrap();
        }

        for rc_param_assignation in &self.parameters {
            let borrowed_param_assignation = rc_param_assignation.read().unwrap();

            let param_assignation_designer = designer
                .add_parameter(&borrowed_param_assignation.name)
                .unwrap();

            borrowed_param_assignation.make_design(&param_assignation_designer)?;
        }

        wrap_logic_error!(designer.validate(), self.text.name.position);

        Ok(())
    }
}

impl AssignativeElement for TreatmentInstanciation {
    fn assignative_element(&self) -> AssignativeElementType {
        AssignativeElementType::Treatment(self)
    }

    fn associated_declarative_element(&self) -> Arc<RwLock<dyn DeclarativeElement>> {
        self.treatment.upgrade().unwrap() as Arc<RwLock<dyn DeclarativeElement>>
    }

    /// Search for an assigned model.

    fn find_assigned_model(&self, name: &str) -> Option<&Arc<RwLock<AssignedModel>>> {
        self.models.iter().find(|&m| m.read().unwrap().name == name)
    }

    /// Search for a parameter.
    fn find_assigned_parameter(&self, name: &str) -> Option<&Arc<RwLock<AssignedParameter>>> {
        self.parameters
            .iter()
            .find(|&a| a.read().unwrap().name == name)
    }
}

impl Node for TreatmentInstanciation {
    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {
        let mut children: Vec<Arc<RwLock<dyn Node>>> = Vec::new();

        self.models
            .iter()
            .for_each(|m| children.push(Arc::clone(&m) as Arc<RwLock<dyn Node>>));
        self.parameters
            .iter()
            .for_each(|p| children.push(Arc::clone(&p) as Arc<RwLock<dyn Node>>));

        children
    }

    fn make_references(&mut self, path: &Path) -> Result<(), ScriptError> {
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
                let treatment = borrowed_script.find_treatment(&reference.name);
                if treatment.is_some() {
                    self.type_identifier = path.to_identifier(&reference.name);

                    self.r#type = RefersTo::Treatment(Reference {
                        name: reference.name.clone(),
                        reference: Some(Arc::downgrade(treatment.unwrap())),
                    });
                } else {
                    return Err(ScriptError::semantic(
                        "'".to_string() + &reference.name + "' is unknown.",
                        self.text.r#type.position,
                    ));
                }
            }
        }

        Ok(())
    }
}
