
//! Module dedicated to Treatment semantic analysis.

use super::common::Node;

use std::sync::{Arc, Weak, RwLock};
use crate::script::error::ScriptError;
use crate::script::path::Path;
use crate::script::text::Instanciation as TextTreatment;
use crate::logic::descriptor::identifier::Identifier;
use crate::logic::designer::TreatmentDesigner;

use super::r#use::Use;
use super::sequence::Sequence;
use super::common::Reference;
use super::assignative_element::{AssignativeElement, AssignativeElementType};
use super::assigned_model::AssignedModel;
use super::assigned_parameter::AssignedParameter;
use super::declarative_element::DeclarativeElement;

/// Structure managing and describing semantic of a treatment.
/// 
/// It owns the whole [text instanciation](../../text/instanciation/struct.Instanciation.html).
#[derive(Debug)]
pub struct Treatment {
    pub text: TextTreatment,

    pub sequence: Weak<RwLock<Sequence>>,

    pub name: String,
    pub r#type: RefersTo,
    pub models: Vec<Arc<RwLock<AssignedModel>>>,
    pub parameters: Vec<Arc<RwLock<AssignedParameter>>>,

    pub type_identifier: Option<Identifier>,
}

/// Enumeration managing what treatment type refers to.
/// 
/// This is a convenience enum, as a treatment type may refer either on a [Use](../use/struct.Use.html) or a [Sequence](../sequence/struct.Sequence.html).
/// The `Unknown` variant is aimed to hold a reference-to-nothing, as long as `make_references() hasn't been called.
#[derive(Debug)]
pub enum RefersTo {
    Unkown(Reference<()>),
    Use(Reference<Use>),
    Sequence(Reference<Sequence>),
}

impl Treatment {
    /// Create a new semantic treatment, based on textual treatment.
    /// 
    /// * `sequence`: the parent sequence that "owns" this treatment.
    /// * `text`: the textual treatment.
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
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// // Internally, Script::new call Sequence::new(Arc::clone(&script), text_sequence),
    /// // which will itself call Treatment::new(Arc::clone(&sequence), text_treatment).
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("HPCP").unwrap().read().unwrap();
    /// let borrowed_treatment = borrowed_sequence.find_treatment("CoreSpectralPeaks").unwrap().read().unwrap();
    /// 
    /// assert_eq!(borrowed_treatment.name, "CoreSpectralPeaks");
    /// assert_eq!(borrowed_treatment.parameters.len(), 6);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(sequence: Arc<RwLock<Sequence>>, text: TextTreatment) -> Result<Arc<RwLock<Self>>, ScriptError> {

        let treatment = Arc::<RwLock<Self>>::new(RwLock::new(Self {
            text: text.clone(),
            sequence: Arc::downgrade(&sequence),
            name: text.name.string.clone(),
            r#type: RefersTo::Unkown(Reference::new(text.r#type.string)),
            models: Vec::new(),
            parameters: Vec::new(),
            type_identifier: None,
        }));

        {
            let borrowed_sequence = sequence.read().unwrap();

            let treatment = borrowed_sequence.find_treatment(&text.name.string);
            if treatment.is_some() {
                return Err(ScriptError::semantic("Treatment '".to_string() + &text.name.string + "' is already declared.", text.name.position))
            }
        }

        for m in text.configuration {
            let assigned_model = AssignedModel::new(Arc::clone(&treatment) as Arc<RwLock<dyn AssignativeElement>>, m)?;
            treatment.write().unwrap().models.push(assigned_model);
        }

        for p in text.parameters {
            let assigned_parameter = AssignedParameter::new(Arc::clone(&treatment) as Arc<RwLock<dyn AssignativeElement>>, p)?;
            treatment.write().unwrap().parameters.push(assigned_parameter);
        }

        Ok(treatment)
    }

    pub fn make_design(&self, designer: &Arc<RwLock<TreatmentDesigner>>) -> Result<(), ScriptError> {

        let mut designer = designer.write().unwrap();

        for rc_model_assignation in &self.models {

            let borrowed_model_assignation = rc_model_assignation.read().unwrap();

            designer.add_model(&borrowed_model_assignation.name, &borrowed_model_assignation.model.name).unwrap();
        }

        for rc_param_assignation in &self.parameters {

            let borrowed_param_assignation = rc_param_assignation.read().unwrap();

            let param_assignation_designer = designer.add_parameter(&borrowed_param_assignation.name).unwrap();

            borrowed_param_assignation.make_design(&param_assignation_designer).unwrap();
        }

        designer.validate().unwrap();

        Ok(())

    }
}

impl AssignativeElement for Treatment {

    fn assignative_element(&self) -> AssignativeElementType {
        AssignativeElementType::Treatment(&self)
    }

    fn associated_declarative_element(&self) -> Arc<RwLock<dyn DeclarativeElement>> {
        self.sequence.upgrade().unwrap() as Arc<RwLock<dyn DeclarativeElement>>
    }

    /// Search for an assigned model.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script as TextScript;
    /// # use melodium::script::semantic::script::Script;
    /// # use melodium::script::semantic::assignative_element::AssignativeElement;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("ReadAudioFiles").unwrap().read().unwrap();
    /// let borrowed_treatment = borrowed_sequence.find_treatment("Decoder").unwrap().read().unwrap();
    /// 
    /// let audio_manager = borrowed_treatment.find_assigned_model("AudioManager");
    /// let dont_exist = borrowed_treatment.find_assigned_model("DontExist");
    /// assert!(audio_manager.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    fn find_assigned_model(&self, name: & str) -> Option<&Arc<RwLock<AssignedModel>>> {
        self.models.iter().find(|&m| m.read().unwrap().name == name)
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
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("HPCP").unwrap().read().unwrap();
    /// let borrowed_treatment = borrowed_sequence.find_treatment("CoreSpectralPeaks").unwrap().read().unwrap();
    /// 
    /// let magnitude_threshold = borrowed_treatment.find_assigned_parameter("magnitudeThreshold");
    /// let dont_exist = borrowed_treatment.find_assigned_parameter("dontExist");
    /// assert!(magnitude_threshold.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    fn find_assigned_parameter(&self, name: & str) -> Option<&Arc<RwLock<AssignedParameter>>> {
        self.parameters.iter().find(|&a| a.read().unwrap().name == name)
    }
}

impl Node for Treatment {
    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {

        let mut children: Vec<Arc<RwLock<dyn Node>>> = Vec::new();

        self.models.iter().for_each(|m| children.push(Arc::clone(&m) as Arc<RwLock<dyn Node>>));
        self.parameters.iter().for_each(|p| children.push(Arc::clone(&p) as Arc<RwLock<dyn Node>>));

        children
    }

    fn make_references(&mut self, path: &Path) -> Result<(), ScriptError> {

        if let RefersTo::Unkown(reference) = &self.r#type {

            let rc_sequence = self.sequence.upgrade().unwrap();
            let borrowed_sequence = rc_sequence.read().unwrap();
            let rc_script = borrowed_sequence.script.upgrade().unwrap();
            let borrowed_script = rc_script.read().unwrap();

            let r#use = borrowed_script.find_use(&reference.name);
            if r#use.is_some() {

                let r#use = r#use.unwrap();

                self.type_identifier = r#use.read().unwrap().identifier.clone();

                self.r#type = RefersTo::Use(Reference{
                    name: reference.name.clone(),
                    reference: Some(Arc::downgrade(r#use))
                });
            }
            else {
                let sequence = borrowed_script.find_sequence(&reference.name);
                if sequence.is_some() {

                    self.type_identifier = path.to_identifier(&reference.name);

                    self.r#type = RefersTo::Sequence(Reference{
                        name: reference.name.clone(),
                        reference: Some(Arc::downgrade(sequence.unwrap()))
                    });
                }
                else {
                    return Err(ScriptError::semantic("'".to_string() + &reference.name + "' is unkown.", self.text.r#type.position))
                }
            }
        }

        Ok(())
    }
}
