
use std::fmt::*;
use std::collections::HashMap;
use std::sync::{Arc, Weak, RwLock};
use std::iter::FromIterator;
use super::identified::Identified;
use super::identifier::Identifier;
use super::documented::Documented;
use super::parameterized::Parameterized;
use super::buildable::Buildable;
use super::input::Input;
use super::output::Output;
use super::core_model::CoreModel;
use super::parameter::Parameter;
use super::requirement::Requirement;
use super::treatment::Treatment;
use super::super::builder::Builder;
use super::super::builder::SourceBuilder;
use crate::logic::designer::SequenceDesigner;

#[derive(Debug)]
pub struct CoreSource {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    models: HashMap<String, Arc<CoreModel>>,
    outputs: HashMap<String, Output>,
    source_from: HashMap<Arc<CoreModel>, Vec<String>>,
    builder: Arc<Box<dyn Builder>>,
    auto_reference: Weak<Self>,
}

impl CoreSource {
    pub fn new(
        identifier: Identifier,
        documentation: String,
        models: Vec<(String, Arc<CoreModel>)>,
        source_from: HashMap<Arc<CoreModel>, Vec<String>>,
        outputs: Vec<Output>
    ) -> Arc<Self> {
        #[cfg(not(feature = "doc"))]
        let _ = documentation;
        Arc::new_cyclic(|me| Self {
            identifier,
            #[cfg(feature = "doc")]
            documentation,
            models: HashMap::from_iter(models.iter().map(|m| (m.0.to_string(), Arc::clone(&m.1)))),
            outputs: HashMap::from_iter(outputs.iter().map(|o| (o.name().to_string(), o.clone()))),
            source_from,
            builder: Arc::new(Box::new(SourceBuilder::new(me.clone() as Weak<dyn Treatment>))),
            auto_reference: me.clone(),
        })
    }
}

impl Identified for CoreSource {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl Documented for CoreSource {
    fn documentation(&self) -> &str {
        #[cfg(feature = "doc")]
        {&self.documentation}
        #[cfg(not(feature = "doc"))]
        {&""}
    }
}

impl Parameterized for CoreSource {

    fn parameters(&self) -> &HashMap<String, Parameter> {

        lazy_static! {
            static ref EMPTY_HASHMAP: HashMap<String, Parameter> = HashMap::new();
        }

        &EMPTY_HASHMAP
    }

    fn as_parameterized(&self) -> Arc<dyn Parameterized> {
        self.auto_reference.upgrade().unwrap()
    }
}

impl Buildable for CoreSource {

    fn builder(&self) -> Arc<Box<dyn Builder>> {
        Arc::clone(&self.builder)
    }
}

impl Treatment for CoreSource {

    fn inputs(&self) -> &HashMap<String, Input> {

        lazy_static! {
            static ref EMPTY_HASHMAP: HashMap<String, Input> = HashMap::new();
        }

        &EMPTY_HASHMAP
    }

    fn outputs(&self) -> &HashMap<String, Output> {
        &self.outputs
    }

    fn models(&self) -> &HashMap<String, Arc<CoreModel>> {
        &self.models
    }

    fn requirements(&self) -> &HashMap<String, Requirement> {
        
        lazy_static! {
            static ref EMPTY_HASHMAP: HashMap<String, Requirement> = HashMap::new();
        }

        &EMPTY_HASHMAP
    }

    fn source_from(&self) -> &HashMap<Arc<CoreModel>, Vec<String>> {
        &self.source_from
    }

    fn designer(&self) -> Option<Arc<RwLock<SequenceDesigner>>> {
        None
    }

    fn as_buildable(&self) -> Arc<dyn Buildable> {
        self.auto_reference.upgrade().unwrap()
    }
}

impl Display for CoreSource {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {

        writeln!(f, "Treatment source `{}`", self.identifier.to_string())?;

        if !self.models.is_empty() {
            writeln!(f, "\nModels:")?;

            for model in &self.models {
                writeln!(f, "- _{}_: `{}`", model.0, model.1.identifier().to_string())?;
            }
        }

        if !self.outputs.is_empty() {
            writeln!(f, "\nOutputs:")?;

            for output in &self.outputs {
                writeln!(f, "- {}", output.1)?;
            }
        }

        Ok(())
        
    }
}

