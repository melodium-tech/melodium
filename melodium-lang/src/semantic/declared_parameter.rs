//! Module dedicated to DeclaredParameter semantic analysis.

use super::common::Node;
use super::declarative_element::{DeclarativeElement, DeclarativeElementType};
use super::r#type::Type;
use super::value::Value;
use super::variability::Variability;
use crate::error::ScriptError;
use crate::text::Parameter as TextParameter;
use melodium_common::descriptor::{Flow as FlowDescriptor, Parameter as ParameterDescriptor};
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of a declared parameter.
///
/// A _declared_ parameter is a parameter for which name and type are expected, as well as an optionnal value.
/// It is used by [Treatments](super::Treatment) and [Models](super::Model).
///
/// It owns the whole [text parameter](crate::text::Parameter).
#[derive(Debug)]
pub struct DeclaredParameter {
    pub text: TextParameter,

    pub parent: Weak<RwLock<dyn DeclarativeElement>>,

    pub name: String,
    pub variability: Variability,
    pub r#type: Type,
    pub value: Option<Arc<RwLock<Value>>>,
}

impl DeclaredParameter {
    /// Create a new semantic declared parameter, based on textual parameter.
    ///
    /// * `parent`: the parent element owning this declared parameter.
    /// * `text`: the textual parameter.
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
    /// # use melodium::script::semantic::declarative_element::DeclarativeElement;
    /// # use melodium::script::semantic::r#type::{TypeName, TypeStructure};
    /// let address = "melodium-tests/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    ///
    /// let text_script = TextScript::build(&raw_text)?;
    ///
    /// let script = Script::new(text_script)?;
    /// // Internally, Script::new call Treatment::new(Rc::clone(&script), text_treatment),
    /// // which will itself call DeclaredParameter::new(Rc::clone(&treatment), text_parameter).
    ///
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_treatment = borrowed_script.find_treatment("AudioToHpcpImage").unwrap().read().unwrap();
    /// let borrowed_declared_parameter = borrowed_treatment.find_declared_parameter("hopSize").unwrap().read().unwrap();
    ///
    /// assert_eq!(borrowed_declared_parameter.name, "hopSize");
    /// assert_eq!(borrowed_declared_parameter.r#type.structure, TypeStructure::Scalar);
    /// assert_eq!(borrowed_declared_parameter.r#type.name, TypeName::U64);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(
        parent: Arc<RwLock<dyn DeclarativeElement>>,
        text: TextParameter,
    ) -> Result<Arc<RwLock<Self>>, ScriptError> {
        let variability;
        let r#type;
        let value;
        {
            let borrowed_parent = parent.read().unwrap();

            let parameter = borrowed_parent.find_declared_parameter(&text.name.string);
            if parameter.is_some() {
                return Err(ScriptError::semantic(
                    "Parameter '".to_string() + &text.name.string + "' is already declared.",
                    text.name.position,
                ));
            }

            match borrowed_parent.declarative_element() {
                DeclarativeElementType::Model(_) => {
                    if let Some(text_variability) = &text.variability {
                        variability = Variability::from_string(&text_variability.string).unwrap();
                        if variability != Variability::Const {
                            return Err(ScriptError::semantic(
                                "Parameter '".to_string()
                                    + &text.name.string
                                    + "' cannot be variable (const required for models).",
                                text.name.position,
                            ));
                        }
                    } else {
                        variability = Variability::Const;
                    }
                }
                DeclarativeElementType::Treatment(_) => {
                    if let Some(text_variability) = &text.variability {
                        variability = Variability::from_string(&text_variability.string).unwrap();
                    } else {
                        variability = Variability::Var;
                    }
                }
            }

            if text.r#type.is_none() {
                return Err(ScriptError::semantic(
                    "Parameter '".to_string() + &text.name.string + "' do not have type.",
                    text.name.position,
                ));
            }
            r#type = Type::new(text.r#type.as_ref().unwrap().clone())?;

            if text.value.is_some() {
                value = Some(Value::new(
                    Arc::clone(&parent),
                    text.value.as_ref().unwrap().clone(),
                )?);
            } else {
                value = None;
            }
        }

        Ok(Arc::<RwLock<Self>>::new(RwLock::new(Self {
            parent: Arc::downgrade(&parent),
            name: text.name.string.clone(),
            text,
            variability,
            r#type,
            value,
        })))
    }

    pub fn make_descriptor(&self) -> Result<ParameterDescriptor, ScriptError> {
        let (datatype, flow) = self.r#type.make_descriptor()?;
        if flow != FlowDescriptor::Block {
            return Err(ScriptError::semantic(
                "Parameter '".to_string() + &self.text.name.string + "' cannot have flow.",
                self.text.name.position,
            ));
        }

        let value = if let Some(val) = &self.value {
            Some(val.read().unwrap().make_executive_value(&datatype)?)
        } else {
            None
        };

        let parameter = ParameterDescriptor::new(
            &self.name,
            self.variability.to_descriptor(),
            datatype,
            value,
        );

        Ok(parameter)
    }
}

impl Node for DeclaredParameter {}
