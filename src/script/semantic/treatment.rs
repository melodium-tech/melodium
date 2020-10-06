
//! Module dedicated to Treatment semantic analysis.

use super::common::Node;

use std::rc::{Rc, Weak};
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Instanciation as TextTreatment;

use super::r#use::Use;
use super::sequence::Sequence;
use super::common::Reference;
use super::assignative_element::{AssignativeElement, AssignativeElementType};
use super::assigned_model::AssignedModel;
use super::assigned_parameter::AssignedParameter;
use super::declarative_element::DeclarativeElement;

/// Structure managing and describing semantic of a treatment.
/// 
/// It owns the whole [text treatment](../../text/treatment/struct.Treatment.html).
pub struct Treatment {
    pub text: TextTreatment,

    pub sequence: Weak<RefCell<Sequence>>,

    pub name: String,
    pub r#type: RefersTo,
    pub models: Vec<Rc<RefCell<AssignedModel>>>,
    pub parameters: Vec<Rc<RefCell<AssignedParameter>>>,
}

/// Enumeration managing what treatment type refers to.
/// 
/// This is a convenience enum, as a treatment type may refer either on a [Use](../use/struct.Use.html) or a [Sequence](../sequence/struct.Sequence.html).
/// The `Unknown` variant is aimed to hold a reference-to-nothing, as long as `make_references() hasn't been called.
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
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// // Internally, Script::new call Sequence::new(Rc::clone(&script), text_sequence),
    /// // which will itself call Treatment::new(Rc::clone(&sequence), text_treatment).
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_sequence = borrowed_script.find_sequence("HPCP").unwrap().borrow();
    /// let borrowed_treatment = borrowed_sequence.find_treatment("CoreSpectralPeaks").unwrap().borrow();
    /// 
    /// assert_eq!(borrowed_treatment.name, "CoreSpectralPeaks");
    /// assert_eq!(borrowed_treatment.parameters.len(), 6);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(sequence: Rc<RefCell<Sequence>>, text: TextTreatment) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let treatment = Rc::<RefCell<Self>>::new(RefCell::new(Self {
            text: text.clone(),
            sequence: Rc::downgrade(&sequence),
            name: text.name.string.clone(),
            r#type: RefersTo::Unkown(Reference::new(text.r#type.string)),
            models: Vec::new(),
            parameters: Vec::new(),
        }));

        {
            let borrowed_sequence = sequence.borrow();

            let treatment = borrowed_sequence.find_treatment(&text.name.string);
            if treatment.is_some() {
                return Err(ScriptError::semantic("Treatment '".to_string() + &text.name.string + "' is already declared.", text.name.position))
            }
        }

        for m in text.configuration {
            let assigned_model = AssignedModel::new(Rc::clone(&treatment) as Rc<RefCell<dyn AssignativeElement>>, m)?;
            treatment.borrow_mut().models.push(assigned_model);
        }

        for p in text.parameters {
            let assigned_parameter = AssignedParameter::new(Rc::clone(&treatment) as Rc<RefCell<dyn AssignativeElement>>, p)?;
            treatment.borrow_mut().parameters.push(assigned_parameter);
        }

        Ok(treatment)
    }
}

impl AssignativeElement for Treatment {

    fn assignative_element(&self) -> AssignativeElementType {
        AssignativeElementType::Treatment(&self)
    }

    fn associated_declarative_element(&self) -> Rc<RefCell<dyn DeclarativeElement>> {
        self.sequence.upgrade().unwrap() as Rc<RefCell<dyn DeclarativeElement>>
    }

    /// Search for an assigned model.
    fn find_assigned_model(&self, name: & str) -> Option<&Rc<RefCell<AssignedModel>>> {
        self.models.iter().find(|&m| m.borrow().name == name)
    }

    /// Search for a parameter.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// # use melodium_rust::script::semantic::assignative_element::AssignativeElement;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_sequence = borrowed_script.find_sequence("HPCP").unwrap().borrow();
    /// let borrowed_treatment = borrowed_sequence.find_treatment("CoreSpectralPeaks").unwrap().borrow();
    /// 
    /// let magnitude_threshold = borrowed_treatment.find_assigned_parameter("magnitudeThreshold");
    /// let dont_exist = borrowed_treatment.find_assigned_parameter("dontExist");
    /// assert!(magnitude_threshold.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    fn find_assigned_parameter(&self, name: & str) -> Option<&Rc<RefCell<AssignedParameter>>> {
        self.parameters.iter().find(|&a| a.borrow().name == name)
    }
}

impl Node for Treatment {
    fn children(&self) -> Vec<Rc<RefCell<dyn Node>>> {

        let mut children: Vec<Rc<RefCell<dyn Node>>> = Vec::new();

        self.models.iter().for_each(|m| children.push(Rc::clone(&m) as Rc<RefCell<dyn Node>>));
        self.parameters.iter().for_each(|p| children.push(Rc::clone(&p) as Rc<RefCell<dyn Node>>));

        children
    }

    fn make_references(&mut self) -> Result<(), ScriptError> {

        if let RefersTo::Unkown(reference) = &self.r#type {

            let rc_sequence = self.sequence.upgrade().unwrap();
            let borrowed_sequence = rc_sequence.borrow();
            let rc_script = borrowed_sequence.script.upgrade().unwrap();
            let borrowed_script = rc_script.borrow();

            let r#use = borrowed_script.find_use(&reference.name);
            if r#use.is_some() {

                self.r#type = RefersTo::Use(Reference{
                    name: reference.name.clone(),
                    reference: Some(Rc::downgrade(r#use.unwrap()))
                });
            }
            else {
                let sequence = borrowed_script.find_sequence(&reference.name);
                if sequence.is_some() {

                    self.r#type = RefersTo::Sequence(Reference{
                        name: reference.name.clone(),
                        reference: Some(Rc::downgrade(sequence.unwrap()))
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
