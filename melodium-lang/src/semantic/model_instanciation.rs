//! Module dedicated to InstanciedModel semantic analysis.

use super::assignative_element::{AssignativeElement, AssignativeElementType};
use super::assigned_parameter::AssignedParameter;
use super::common::Node;
use super::common::Reference;
use super::declarative_element::DeclarativeElement;
use super::model::Model;
use super::r#use::Use;
use super::treatment::Treatment;
use crate::error::{wrap_logic_error, ScriptError};
use crate::path::Path;
use crate::text::Instanciation as TextInstanciation;
use melodium_common::descriptor::Identifier;
use melodium_engine::designer::ModelInstanciation as ModelInstanciationDesigner;
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of a model instanciation.
///
/// It owns the whole [text instanciation](../../text/instanciation/struct.Instanciation.html).
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
/// This is a convenience enum, as a model instanciation may refer either on a [Use](../use/struct.Use.html) or a [Model](../model/struct.Model.html).
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
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](../common/trait.Node.html).
    ///
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script as TextScript;
    /// # use melodium::script::semantic::script::Script;
    /// let address = "melodium-tests/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    ///
    /// let text_script = TextScript::build(&raw_text)?;
    ///
    /// let script = Script::new(text_script)?;
    /// // Internally, Script::new call Treatment::new(Arc::clone(&script), text_treatment),
    /// // which will itself call InstanciedModel::new(Arc::clone(&treatment), text_instanciation).
    ///
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_treatment = borrowed_script.find_treatment("Main").unwrap().read().unwrap();
    /// let borrowed_instancied_model = borrowed_treatment.find_instancied_model("Files").unwrap().read().unwrap();
    ///
    /// assert_eq!(borrowed_instancied_model.name, "Files");
    /// assert_eq!(borrowed_instancied_model.parameters.len(), 1);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(
        treatment: Arc<RwLock<Treatment>>,
        text: TextInstanciation,
    ) -> Result<Arc<RwLock<Self>>, ScriptError> {
        let model = Arc::<RwLock<Self>>::new(RwLock::new(Self {
            text: text.clone(),
            treatment: Arc::downgrade(&treatment),
            name: text.name.string.clone(),
            r#type: RefersTo::Unknown(Reference::new(text.r#type.string)),
            parameters: Vec::new(),
            type_identifier: None,
        }));

        {
            let borrowed_treatment = treatment.read().unwrap();

            if let Some(_) = borrowed_treatment.find_model_instanciation(&text.name.string) {
                return Err(ScriptError::semantic(
                    "Model '".to_string() + &text.name.string + "' is already instancied.",
                    text.name.position,
                ));
            }
        }

        for p in text.parameters {
            let assigned_parameter = AssignedParameter::new(
                Arc::clone(&model) as Arc<RwLock<dyn AssignativeElement>>,
                p,
            )?;
            model.write().unwrap().parameters.push(assigned_parameter);
        }

        Ok(model)
    }

    pub fn make_design(
        &self,
        designer: &Arc<RwLock<ModelInstanciationDesigner>>,
    ) -> Result<(), ScriptError> {
        let mut designer = designer.write().unwrap();

        for rc_assignation in &self.parameters {
            let borrowed_assignation = rc_assignation.read().unwrap();

            let assignation_designer = wrap_logic_error!(
                designer.add_parameter(&borrowed_assignation.name),
                borrowed_assignation.text.name.position
            );

            borrowed_assignation.make_design(&assignation_designer)?;
        }

        wrap_logic_error!(designer.validate(), self.text.name.position);

        Ok(())
    }
}

impl AssignativeElement for ModelInstanciation {
    fn assignative_element(&self) -> AssignativeElementType {
        AssignativeElementType::ModelInstanciation(self)
    }

    fn associated_declarative_element(&self) -> Arc<RwLock<dyn DeclarativeElement>> {
        self.treatment.upgrade().unwrap() as Arc<RwLock<dyn DeclarativeElement>>
    }

    /// Search for a parameter.
    ///
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script as TextScript;
    /// # use melodium::script::semantic::script::Script;
    /// # use melodium::script::semantic::assignative_element::AssignativeElement;
    /// let address = "melodium-tests/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    ///
    /// let text_script = TextScript::build(&raw_text)?;
    ///
    /// let script = Script::new(text_script)?;
    ///
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_treatment = borrowed_script.find_treatment("Main").unwrap().read().unwrap();
    /// let borrowed_instancied_model = borrowed_treatment.find_instancied_model("Files").unwrap().read().unwrap();
    ///
    /// let directory = borrowed_instancied_model.find_assigned_parameter("directory");
    /// let dont_exist = borrowed_instancied_model.find_assigned_parameter("dontExist");
    /// assert!(directory.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
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
                let model = borrowed_script.find_model(&reference.name);
                if model.is_some() {
                    self.type_identifier = path.to_identifier(&reference.name);

                    self.r#type = RefersTo::Model(Reference {
                        name: reference.name.clone(),
                        reference: Some(Arc::downgrade(model.unwrap())),
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
