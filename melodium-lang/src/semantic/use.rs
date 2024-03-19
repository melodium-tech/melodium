//! Module dedicated to Use semantic analysis.

use super::common::Node;
use super::script::Script;
use crate::text::Use as TextUse;
use crate::ScriptError;
use crate::{path::Path, ScriptResult};
use melodium_common::descriptor::{Identifier, IdentifierRequirement, Version, VersionReq};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of a use.
///
/// It owns the whole [text use](Use).
#[derive(Debug, Clone)]
pub struct Use {
    pub text: TextUse,

    pub script: Weak<RwLock<Script>>,

    pub path: Path,
    pub element: String,
    pub r#as: String,

    pub identifier: Option<IdentifierRequirement>,
}

impl Use {
    /// Create a new semantic use, based on textual use.
    ///
    /// * `script`: the parent script that "owns" this use.
    /// * `text`: the textual use.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    ///
    pub fn new(
        script: Arc<RwLock<Script>>,
        text: TextUse,
        version: Version,
    ) -> ScriptResult<Arc<RwLock<Self>>> {
        let mut result = ScriptResult::new_success(());

        let r#as;
        if let Some(ps) = text.r#as.clone() {
            r#as = ps;
        } else {
            r#as = text.element.clone();
        }

        {
            let borrowed_script = script.read().unwrap();

            let r#use = borrowed_script.find_use(&r#as.string);
            if r#use.is_some() {
                result = result.and_degrade_failure(ScriptResult::new_failure(
                    ScriptError::already_used_name(108, r#as.clone()),
                ));
            }
        }

        let path = Path::new(
            version,
            text.path.iter().map(|i| i.string.clone()).collect(),
        );

        result.and_then(|_| {
            ScriptResult::new_success(Arc::<RwLock<Self>>::new(RwLock::new(Self {
                script: Arc::downgrade(&script),
                path,
                element: text.element.string.clone(),
                r#as: r#as.string.clone(),
                text,
                identifier: None,
            })))
        })
    }
}

impl Node for Use {
    fn make_references(
        &mut self,
        path: &Path,
        versions: &HashMap<String, VersionReq>,
    ) -> ScriptResult<()> {
        if !self.path.is_valid() {
            ScriptResult::new_failure(ScriptError::invalid_root(
                107,
                self.text.element.clone(),
                self.path.root(),
            ))
        } else {
            match self.path.root().as_str() {
                "root" => {
                    // "Root" package case

                    let mut steps = vec![path.root()];
                    self.path
                        .path()
                        .iter()
                        .skip(1)
                        .for_each(|s| steps.push(s.clone()));

                    self.identifier = Some(
                        (&Identifier::new_versionned(path.version(), steps, &self.element)).into(),
                    );

                    ScriptResult::new_success(())
                }
                "local" => {
                    // "Local" case

                    let mut steps = path.path().clone();
                    self.path
                        .path()
                        .iter()
                        .skip(1)
                        .for_each(|s| steps.push(s.clone()));

                    self.identifier = Some(
                        (&Identifier::new_versionned(path.version(), steps, &self.element)).into(),
                    );

                    ScriptResult::new_success(())
                }
                _ => {
                    // "Non-local" case

                    if let Some(version_req) = versions.get(&self.path.root()).cloned() {
                        self.identifier = Some(IdentifierRequirement::new(
                            version_req,
                            self.path.path().clone(),
                            &self.element,
                        ));
                        ScriptResult::new_success(())
                    } else {
                        ScriptResult::new_failure(ScriptError::unexisting_dependency(
                            184,
                            self.text.element.clone(),
                            self.path.root(),
                        ))
                    }
                }
            }
        }
    }
}
