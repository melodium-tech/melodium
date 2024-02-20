//! Module dedicated to Model semantic analysis.

use super::assignative_element::{AssignativeElement, AssignativeElementType};
use super::assigned_parameter::AssignedParameter;
use super::common::Node;
use super::common::Reference;
use super::declarative_element::{DeclarativeElement, DeclarativeElementType};
use super::declared_parameter::DeclaredParameter;
use super::r#use::Use;
use super::script::Script;
use crate::error::ScriptError;
use crate::path::Path;
use crate::text::Model as TextModel;
use crate::ScriptResult;
use melodium_common::descriptor::{Collection, Entry, Identifier, VersionReq};
use melodium_engine::descriptor::Model as ModelDescriptor;
use melodium_engine::designer::Model as ModelDesigner;
use melodium_engine::LogicError;
use std::collections::HashMap;
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
    pub fn new(script: Arc<RwLock<Script>>, text: TextModel) -> ScriptResult<Arc<RwLock<Self>>> {
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
        let mut result = ScriptResult::new_success(model.clone());

        {
            let borrowed_script = script.read().unwrap();

            let model = borrowed_script.find_model(&text.name.string);
            if model.is_some() {
                result = result.and_degrade_failure(ScriptResult::new_failure(
                    ScriptError::already_used_name(122, text.name.clone()),
                ));
            }

            let r#use = borrowed_script.find_use(&text.name.string);
            if r#use.is_some() {
                result = result.and_degrade_failure(ScriptResult::new_failure(
                    ScriptError::already_used_name(123, text.name),
                ));
            }
        }

        for p in text.parameters {
            if let Some(declared_parameter) = result.merge_degrade_failure(DeclaredParameter::new(
                Arc::clone(&model) as Arc<RwLock<dyn DeclarativeElement>>,
                p,
            )) {
                model.write().unwrap().parameters.push(declared_parameter);
            }
        }

        for a in text.assignations {
            if let Some(assigned_parameter) = result.merge_degrade_failure(AssignedParameter::new(
                Arc::clone(&model) as Arc<RwLock<dyn AssignativeElement>>,
                a,
            )) {
                model.write().unwrap().assignations.push(assigned_parameter);
            }
        }

        result
    }

    pub fn make_descriptor(&self, collection: &Collection) -> ScriptResult<Arc<ModelDescriptor>> {
        let (ref type_identifier, _position) = match &self.r#type {
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
                    .into(),
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
            _ => {
                return ScriptResult::new_failure(ScriptError::reference_unset(
                    124,
                    format!("{:?}", &self.r#type),
                ))
            }
        };

        if let Some(Entry::Model(base_descriptor)) = collection.get(type_identifier) {
            let mut result = ScriptResult::new_success(());

            let mut descriptor =
                ModelDescriptor::new(self.identifier.as_ref().unwrap().clone(), base_descriptor);

            if let Some(annotations) = self.text.annotations.as_ref() {
                if let Some(doc) = &annotations.doc {
                    descriptor.set_documentation(&doc.string);
                }

                for annotation in &annotations.annotations {
                    if let Some((name, attribute)) = annotation.as_attribute() {
                        descriptor.add_attribute(name, attribute);
                    }
                }
            }

            for rc_parameter in &self.parameters {
                let borrowed_parameter = rc_parameter.read().unwrap();
                if let Some(parameter_descriptor) =
                    result.merge_degrade_failure(borrowed_parameter.make_descriptor(collection))
                {
                    descriptor.add_parameter(parameter_descriptor);
                }
            }

            result.and_then(|_| {
                let descriptor = descriptor.commit();

                *self.descriptor.write().unwrap() = Some(descriptor.clone());

                ScriptResult::new_success(descriptor)
            })
        } else {
            ScriptResult::new_failure(
                LogicError::unexisting_model(
                    125,
                    self.identifier.as_ref().unwrap().clone(),
                    type_identifier.clone(),
                    Some(self.text.r#type.into_ref()),
                )
                .into(),
            )
        }
    }

    pub fn make_design(&self, collection: &Arc<Collection>) -> ScriptResult<()> {
        let borrowed_descriptor = self.descriptor.read().unwrap();
        let descriptor = if let Some(descriptor) = &*borrowed_descriptor {
            descriptor
        } else {
            return ScriptResult::new_failure(ScriptError::no_descriptor(
                126,
                self.text.name.clone(),
            ));
        };
        let mut result = ScriptResult::new_success(());

        let rc_designer: Arc<RwLock<ModelDesigner>> = if let Some(designer) = result
            .merge_degrade_failure(ScriptResult::from(
                descriptor.designer(Arc::clone(collection), Some(self.text.name.into_ref())),
            )) {
            designer
        } else {
            return result;
        };

        for rc_assignation in &self.assignations {
            let borrowed_assignation = rc_assignation.read().unwrap();

            let tmp_status = rc_designer.write().unwrap().add_parameter(
                &borrowed_assignation.name,
                Some(borrowed_assignation.text.name.into_ref()),
            );
            if let Some(assignation_designer) =
                result.merge_degrade_failure(ScriptResult::from(tmp_status))
            {
                result = result.and_degrade_failure(
                    borrowed_assignation.make_design(&assignation_designer, collection),
                );
            }
        }

        result = result.and_degrade_failure(ScriptResult::from(descriptor.commit_design()));

        result
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
    fn make_references(
        &mut self,
        path: &Path,
        _versions: &HashMap<String, VersionReq>,
    ) -> ScriptResult<()> {
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
                return ScriptResult::new_failure(ScriptError::unimported_element(
                    127,
                    self.text.name.clone(),
                ));
            }

            self.identifier = path.to_identifier(&self.name);
        }

        ScriptResult::new_success(())
    }
}
