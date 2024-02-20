//! Module dedicated to Requirement semantic analysis.

use super::common::Node;
use super::common::Reference;
use super::r#use::Use;
use super::treatment::Treatment;
use crate::error::ScriptError;
use crate::path::Path;
use crate::text::Requirement as TextRequirement;
use crate::ScriptResult;
use melodium_common::descriptor::IdentifierRequirement;
use melodium_common::descriptor::VersionReq;
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of a requirement.
///
/// It owns the whole [text requirement](TextRequirement).
#[derive(Debug)]
pub struct Requirement {
    pub text: TextRequirement,

    pub treatment: Weak<RwLock<Treatment>>,

    pub name: String,
    pub r#type: RefersTo,

    pub type_identifier: Option<IdentifierRequirement>,
}

/// Enumeration managing what requirement type refers to.
///
/// This is a convenience enum, as a requirement may only refer on a [Use] context.
/// The `Unknown` variant is aimed to hold a reference-to-nothing, as long as `make_references() hasn't been called.
#[derive(Debug)]
pub enum RefersTo {
    Unknown(Reference<()>),
    Use(Reference<Use>),
}

impl Requirement {
    /// Create a new semantic requirement, based on textual requirement.
    ///
    /// * `treatment`: the parent treatment that "owns" this requirement.
    /// * `text`: the textual requirement.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    ///
    pub fn new(
        treatment: Arc<RwLock<Treatment>>,
        text: TextRequirement,
    ) -> ScriptResult<Arc<RwLock<Self>>> {
        {
            let borrowed_treatment = treatment.read().unwrap();

            let requirement = borrowed_treatment.find_requirement(&text.name.string);
            if requirement.is_some() {
                return ScriptResult::new_failure(ScriptError::already_declared(118, text.name));
            }
        }

        ScriptResult::new_success(Arc::<RwLock<Self>>::new(RwLock::new(Self {
            treatment: Arc::downgrade(&treatment),
            name: text.name.string.clone(),
            r#type: RefersTo::Unknown(Reference::new(text.name.string.clone())),
            text,
            type_identifier: None,
        })))
    }
}

impl Node for Requirement {
    fn make_references(
        &mut self,
        _path: &Path,
        _versions: &HashMap<String, VersionReq>,
    ) -> ScriptResult<()> {
        if let RefersTo::Unknown(reference) = &self.r#type {
            let rc_treatment = self.treatment.upgrade().unwrap();
            let borrowed_treatment = rc_treatment.read().unwrap();
            let rc_script = borrowed_treatment.script.upgrade().unwrap();
            let borrowed_script = rc_script.read().unwrap();

            let r#use = borrowed_script.find_use(&reference.name);
            if r#use.is_some() {
                let r#use = r#use.unwrap();

                self.type_identifier = r#use.read().unwrap().identifier.as_ref().cloned();

                self.r#type = RefersTo::Use(Reference {
                    name: reference.name.clone(),
                    reference: Some(Arc::downgrade(r#use)),
                });
            } else {
                return ScriptResult::new_failure(ScriptError::unimported_element(
                    119,
                    self.text.name.clone(),
                ));
            }
        }

        ScriptResult::new_success(())
    }
}
