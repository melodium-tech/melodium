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
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of a declared model.
///
/// It owns optionnally the whole [text parameter](../../text/parameter/struct.Parameter.html),
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
/// This is a convenience enum, as a declared model type may refer either on a [Use](../use/struct.Use.html), a [Model](../use/struct.Model.html), or an [InstanciedModel](../instancied_model/struct.InstanciedModel.html).
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
    /// // which will itself call DeclaredModel::from_instancied_model(Arc::clone(&instancied_model)).
    ///
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_treatment = borrowed_script.find_treatment("Main").unwrap().read().unwrap();
    /// let borrowed_declared_model = borrowed_treatment.find_declared_model("Files").unwrap().read().unwrap();
    ///
    /// assert_eq!(borrowed_declared_model.name, "Files");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn from_instancied_model(
        instancied_model: Arc<RwLock<ModelInstanciation>>,
    ) -> Result<Arc<RwLock<Self>>, ScriptError> {
        let borrowed_instancied_model = instancied_model.read().unwrap();

        let treatment = borrowed_instancied_model.treatment.upgrade().unwrap();
        let name = borrowed_instancied_model.name.clone();

        let declared_model = Self::make(treatment, borrowed_instancied_model.text.name.clone())?;

        declared_model.write().unwrap().refers = RefersTo::InstanciedModel(Reference {
            name: name,
            reference: Some(Arc::downgrade(&instancied_model)),
        });

        Ok(declared_model)
    }

    /// Create a new semantic declaration of model, based on textual parameter.
    ///
    /// * `treatment`: the treatment owning this declaration.
    /// * `text`: the textual model.
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
    /// // which will itself call DeclaredModel::new(Arc::clone(&treatment), text_parameter).
    ///
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_treatment = borrowed_script.find_treatment("AudioToHpcpImage").unwrap().read().unwrap();
    /// let borrowed_declared_model = borrowed_treatment.find_declared_model("AudioManager").unwrap().read().unwrap();
    ///
    /// assert_eq!(borrowed_declared_model.name, "AudioManager");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(
        treatment: Arc<RwLock<Treatment>>,
        text: TextParameter,
    ) -> Result<Arc<RwLock<Self>>, ScriptError> {
        let refers_string;
        if let Some(r#type) = &text.r#type {
            if r#type.first_level_structure.is_some() || r#type.second_level_structure.is_some() {
                return Err(ScriptError::semantic(
                    "Model '".to_string() + &text.name.string + "' cannot have type structure.",
                    text.name.position,
                ));
            }

            refers_string = r#type.name.string.clone();
        } else {
            return Err(ScriptError::semantic(
                "Model '".to_string() + &text.name.string + "' do not have type.",
                text.name.position,
            ));
        }

        if text.value.is_some() {
            return Err(ScriptError::semantic(
                "Model '".to_string() + &text.name.string + "' cannot be assigned to a value.",
                text.name.position,
            ));
        }

        let declared_model = Self::make(treatment, text.name.clone())?;
        {
            let mut borrowed_declared_model = declared_model.write().unwrap();
            borrowed_declared_model.text = Some(text);
            borrowed_declared_model.refers = RefersTo::Unknown(Reference::new(refers_string));
        }

        Ok(declared_model)
    }

    fn make(
        treatment: Arc<RwLock<Treatment>>,
        name: PositionnedString,
    ) -> Result<Arc<RwLock<Self>>, ScriptError> {
        let borrowed_treatment = treatment.read().unwrap();

        let declared_model = borrowed_treatment.find_declared_model(&name.string.clone());
        if declared_model.is_some() {
            return Err(ScriptError::semantic(
                "Model '".to_string() + &name.string.clone() + "' is already declared.",
                name.position.clone(),
            ));
        }

        Ok(Arc::<RwLock<Self>>::new(RwLock::new(Self {
            treatment: Arc::downgrade(&treatment),
            name: name.string.clone(),
            text: None,
            refers: RefersTo::Unknown(Reference::new(name.string)),
        })))
    }

    pub fn comes_from_instancied(&self) -> bool {
        match self.refers {
            RefersTo::InstanciedModel(_) => true,
            _ => false,
        }
    }
}

impl Node for DeclaredModel {
    fn make_references(&mut self, _path: &Path) -> Result<(), ScriptError> {
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
                return Err(ScriptError::semantic(
                    "'".to_string() + &reference.name + "' is unknown.",
                    self.text
                        .as_ref()
                        .unwrap()
                        .r#type
                        .as_ref()
                        .unwrap()
                        .name
                        .position,
                ));
            }
        }

        Ok(())
    }
}
