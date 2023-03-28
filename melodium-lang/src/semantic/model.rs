//! Module dedicated to Model semantic analysis.

use super::assignative_element::{AssignativeElement, AssignativeElementType};
use super::assigned_parameter::AssignedParameter;
use super::common::Node;
use super::common::Reference;
use super::declarative_element::{DeclarativeElement, DeclarativeElementType};
use super::declared_parameter::DeclaredParameter;
use super::r#use::Use;
use super::script::Script;
use crate::error::{wrap_logic_error, ScriptError};
use crate::path::Path;
use crate::text::Model as TextModel;
use melodium_common::descriptor::{Collection, Entry, Identifier, Model as ModelTrait};
use melodium_engine::descriptor::Model as ModelDescriptor;
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of a model.
///
/// It owns the whole [text model](TextModel).
#[derive(Debug)]
pub struct Model {
    pub text: TextModel,

    pub script: Weak<RwLock<Script>>,

    pub name: String,
    pub parameters: Vec<Arc<RwLock<DeclaredParameter>>>,
    pub r#type: RefersTo,
    pub assignations: Vec<Arc<RwLock<AssignedParameter>>>,

    pub identifier: Option<Identifier>,
    pub descriptor: RwLock<Option<Arc<ModelDescriptor>>>,

    auto_reference: Weak<RwLock<Self>>,
}

#[derive(Debug)]
pub enum RefersTo {
    Unknown(Reference<()>),
    Use(Reference<Use>),
    Model(Reference<Model>),
}

impl Model {
    /// Create a new semantic model, based on textual model.
    ///
    /// * `script`: the parent script that "owns" this model.
    /// * `text`: the textual model.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    ///
    pub fn new(
        script: Arc<RwLock<Script>>,
        text: TextModel,
    ) -> Result<Arc<RwLock<Self>>, ScriptError> {
        let model = Arc::<RwLock<Self>>::new_cyclic(|me| {
            RwLock::new(Self {
                text: text.clone(),
                script: Arc::downgrade(&script),
                name: text.name.string.clone(),
                parameters: Vec::new(),
                r#type: RefersTo::Unknown(Reference::new(text.r#type.string.clone())),
                assignations: Vec::new(),
                identifier: None,
                descriptor: RwLock::new(None),
                auto_reference: me.clone(),
            })
        });

        {
            let borrowed_script = script.read().unwrap();

            let model = borrowed_script.find_model(&text.name.string);
            if model.is_some() {
                return Err(ScriptError::semantic(
                    "'".to_string() + &text.name.string + "' is already declared.",
                    text.name.position,
                ));
            }
        }

        for p in text.parameters {
            let declared_parameter = DeclaredParameter::new(
                Arc::clone(&model) as Arc<RwLock<dyn DeclarativeElement>>,
                p,
            )?;
            model.write().unwrap().parameters.push(declared_parameter);
        }

        for a in text.assignations {
            let assigned_parameter = AssignedParameter::new(
                Arc::clone(&model) as Arc<RwLock<dyn AssignativeElement>>,
                a,
            )?;
            model.write().unwrap().assignations.push(assigned_parameter);
        }

        Ok(model)
    }

    pub fn make_descriptor(&self, collection: &mut Collection) -> Result<(), ScriptError> {
        let (type_identifier, position) = match &self.r#type {
            RefersTo::Model(m) => (
                m.reference
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .read()
                    .unwrap()
                    .identifier
                    .as_ref()
                    .unwrap()
                    .clone(),
                m.reference
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .read()
                    .unwrap()
                    .text
                    .name
                    .position,
            ),
            RefersTo::Use(u) => (
                u.reference
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .read()
                    .unwrap()
                    .identifier
                    .as_ref()
                    .unwrap()
                    .clone(),
                u.reference
                    .as_ref()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .read()
                    .unwrap()
                    .text
                    .element
                    .position,
            ),
            _ => panic!("Descriptor cannot be made without type reference being setted up."),
        };

        if let Some(Entry::Model(base_descriptor)) = collection.get(&type_identifier) {
            /*
            if !core_descriptor.is_core_model() {
                // This should be removed once improvement has been made to inherit scripted model types.
                return Err(ScriptError::semantic("Model type '".to_string() + type_identifier.name() + "' is not a core model.", position));
            }*/

            let mut descriptor =
                ModelDescriptor::new(self.identifier.as_ref().unwrap().clone(), base_descriptor);

            if let Some(documentation) = &self.text.doc {
                descriptor.set_documentation(&documentation.string);
            }

            for rc_parameter in &self.parameters {
                let borrowed_parameter = rc_parameter.read().unwrap();
                let parameter_descriptor = borrowed_parameter.make_descriptor()?;

                descriptor.add_parameter(parameter_descriptor);
            }

            let descriptor = descriptor.commit();

            collection.insert(Entry::Model(Arc::clone(&descriptor) as Arc<dyn ModelTrait>));

            *self.descriptor.write().unwrap() = Some(descriptor);

            Ok(())
        } else {
            Err(ScriptError::semantic(
                "Unknown model type '".to_string() + type_identifier.name() + "'.",
                position,
            ))
        }
    }

    pub fn make_design(&self, collection: &Arc<Collection>) -> Result<(), ScriptError> {
        let borrowed_descriptor = self.descriptor.read().unwrap();
        let descriptor = if let Some(descriptor) = &*borrowed_descriptor {
            descriptor
        } else {
            return Err(ScriptError::no_descriptor());
        };

        let rc_designer = descriptor.designer()?;
        rc_designer
            .write()
            .unwrap()
            .set_collection(Arc::clone(collection));

        for rc_assignation in &self.assignations {
            let borrowed_assignation = rc_assignation.read().unwrap();

            let assignation_designer = wrap_logic_error!(
                rc_designer
                    .write()
                    .unwrap()
                    .add_parameter(&borrowed_assignation.name),
                borrowed_assignation.text.name.position
            );

            borrowed_assignation.make_design(&assignation_designer)?;
        }

        wrap_logic_error!(descriptor.commit_design(), self.text.name.position);

        Ok(())
    }
}

impl DeclarativeElement for Model {
    fn declarative_element(&self) -> DeclarativeElementType {
        DeclarativeElementType::Model(&self)
    }

    fn find_declared_parameter(&self, name: &str) -> Option<&Arc<RwLock<DeclaredParameter>>> {
        self.parameters
            .iter()
            .find(|&p| p.read().unwrap().name == name)
    }
}

impl AssignativeElement for Model {
    fn assignative_element(&self) -> AssignativeElementType {
        AssignativeElementType::Model(&self)
    }

    fn associated_declarative_element(&self) -> Arc<RwLock<dyn DeclarativeElement>> {
        self.auto_reference.upgrade().unwrap()
    }

    fn find_assigned_parameter(&self, name: &str) -> Option<&Arc<RwLock<AssignedParameter>>> {
        self.assignations
            .iter()
            .find(|&a| a.read().unwrap().name == name)
    }
}

impl Node for Model {
    fn make_references(&mut self, path: &Path) -> Result<(), ScriptError> {
        if let RefersTo::Unknown(r#type) = &self.r#type {
            let rc_script = self.script.upgrade().unwrap();
            let borrowed_script = rc_script.read().unwrap();

            if let Some(model) = borrowed_script.find_model(&r#type.name) {
                self.r#type = RefersTo::Model(Reference {
                    name: r#type.name.clone(),
                    reference: Some(Arc::downgrade(model)),
                });
            } else if let Some(r#use) = borrowed_script.find_use(&r#type.name) {
                self.r#type = RefersTo::Use(Reference {
                    name: r#type.name.clone(),
                    reference: Some(Arc::downgrade(r#use)),
                });
            } else {
                return Err(ScriptError::semantic(
                    "'".to_string() + &r#type.name + "' is unknown.",
                    self.text.r#type.position,
                ));
            }

            self.identifier = path.to_identifier(&self.name);
        }

        Ok(())
    }
}
