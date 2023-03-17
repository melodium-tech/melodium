
use std::fmt::*;
use std::sync::{Arc, RwLock, Weak};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::hash::{Hash, Hasher};
use super::identified::Identified;
use super::identifier::Identifier;
use super::documented::Documented;
use super::parameterized::Parameterized;
use super::buildable::Buildable;
use super::model::Model;
use super::parameter::Parameter;
use super::context::Context;
use super::super::builder::Builder;
use crate::logic::builder::CoreModelBuilder;
use crate::logic::designer::ModelDesigner;
use crate::executive::model::Model as ExecutiveModel;
use crate::executive::world::World;

macro_rules! model_sources {
    () => {{
        std::collections::HashMap::new()
    }};
    ($(($source:expr;$($context:expr),*)),*) => {{
        let mut map = std::collections::HashMap::new();
        $(map.insert(
            $source.to_string(),
            vec![
                $(std::sync::Arc::clone(Contexts::get($context).unwrap()),)*
            ]
        );)*
        map
    }};
}
pub(crate) use model_sources;

#[derive(Debug)]
pub struct CoreModel {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    parameters: HashMap<String, Parameter>,
    sources: HashMap<String, Vec<Arc<Context>>>,
    builder: Arc<Box<dyn Builder>>,
    auto_reference: Weak<Self>,
}

impl CoreModel {
    pub fn new(
        identifier: Identifier,
        documentation: String,
        parameters: Vec<Parameter>,
        sources: HashMap<String, Vec<Arc<Context>>>,
        new_model: fn(Arc<World>) -> Arc<dyn ExecutiveModel>
    ) -> Arc<Self> {
        #[cfg(not(feature = "doc"))]
        let _ = documentation;
        Arc::new_cyclic(|me| Self {
            identifier,
            #[cfg(feature = "doc")]
            documentation,
            parameters: HashMap::from_iter(parameters.iter().map(|p| (p.name().to_string(), p.clone()))),
            sources,
            builder: Arc::new(Box::new(CoreModelBuilder::new(new_model))),
            auto_reference: me.clone(),
        })
    }
}

impl Hash for CoreModel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identifier.hash(state);
    }
}

impl PartialEq for CoreModel {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl Eq for CoreModel {}

impl Identified for CoreModel {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl Documented for CoreModel {
    fn documentation(&self) -> &str {
        #[cfg(feature = "doc")]
        {&self.documentation}
        #[cfg(not(feature = "doc"))]
        {&""}
    }
}

impl Parameterized for CoreModel {
        
    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }

    fn as_parameterized(&self) -> Arc<dyn Parameterized> {
        self.auto_reference.upgrade().unwrap()
    }
}

impl Buildable for CoreModel {

    fn builder(&self) -> Arc<Box<dyn Builder>> {
        Arc::clone(&self.builder)
    }
}

impl Model for CoreModel {

    fn is_core_model(&self) -> bool {
        true
    }

    fn core_model(&self) -> Arc<CoreModel> {
        self.auto_reference.upgrade().unwrap()
    }

    fn sources(&self) -> &HashMap<String, Vec<Arc<Context>>> {
        &self.sources
    }

    fn designer(&self) -> Option<Arc<RwLock<ModelDesigner>>> {
        None
    }
}

impl Display for CoreModel {
    
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "model {}({})",
            self.identifier.to_string(),
            self.parameters().iter().map(|(_, p)| p.to_string()).collect::<Vec<_>>().join(", "),
        )?;

        Ok(())
        
    }
}
