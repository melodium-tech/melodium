use core::fmt::{Display, Formatter, Result};
use melodium_common::descriptor::{
    Buildable, Context, Documented, Identified, Identifier, Model as ModelDescriptor,
    ModelBuildMode, Parameter, Parameterized,
};
use melodium_common::executive::{Model as ExecutiveModel, World};
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct Model {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    parameters: HashMap<String, Parameter>,
    sources: HashMap<String, Vec<Arc<dyn Context>>>,
    build_fn: fn(Arc<dyn World>) -> Arc<dyn ExecutiveModel>,
    auto_reference: Weak<Self>,
}

impl Model {
    pub fn new(
        identifier: Identifier,
        documentation: String,
        parameters: Vec<Parameter>,
        sources: Vec<(String, Vec<Arc<dyn Context>>)>,
        build_fn: fn(Arc<dyn World>) -> Arc<dyn ExecutiveModel>,
    ) -> Arc<Self> {
        #[cfg(not(feature = "doc"))]
        let _ = documentation;
        Arc::new_cyclic(|me| Self {
            identifier,
            #[cfg(feature = "doc")]
            documentation,
            parameters: HashMap::from_iter(
                parameters.into_iter().map(|p| (p.name().to_string(), p)),
            ),
            sources: HashMap::from_iter(sources.into_iter()),
            build_fn,
            auto_reference: me.clone(),
        })
    }
}

impl Identified for Model {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}

impl Documented for Model {
    fn documentation(&self) -> &str {
        #[cfg(feature = "doc")]
        {
            &self.documentation
        }
        #[cfg(not(feature = "doc"))]
        {
            &""
        }
    }
}

impl Parameterized for Model {
    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }

    fn as_identified(&self) -> Arc<dyn Identified> {
        self.auto_reference.upgrade().unwrap()
    }
}

impl Buildable<ModelBuildMode> for Model {
    fn build_mode(&self) -> ModelBuildMode {
        ModelBuildMode::Compiled(self.build_fn)
    }

    fn make_use(&self, identifier: &Identifier) -> bool {
        self.sources.iter().any(|(_, contextes)| {
            contextes
                .iter()
                .any(|context| context.identifier() == identifier)
        })
    }
}

impl Display for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "model {}({})",
            self.identifier.to_string(),
            self.parameters
                .iter()
                .map(|(_, p)| p.to_string())
                .collect::<Vec<_>>()
                .join(", "),
        )?;

        Ok(())
    }
}

impl ModelDescriptor for Model {
    fn is_core_model(&self) -> bool {
        true
    }

    fn base_model(&self) -> Option<Arc<dyn ModelDescriptor>> {
        None
    }

    fn sources(&self) -> &HashMap<String, Vec<Arc<dyn Context>>> {
        &self.sources
    }

    fn as_identified(&self) -> Arc<dyn Identified> {
        self.auto_reference.upgrade().unwrap()
    }

    fn as_buildable(&self) -> Arc<dyn Buildable<ModelBuildMode>> {
        self.auto_reference.upgrade().unwrap()
    }

    fn as_parameterized(&self) -> Arc<dyn Parameterized> {
        self.auto_reference.upgrade().unwrap()
    }
}
