use core::fmt::{Display, Formatter, Result};
use melodium_common::descriptor::{
    Attribuable, Attributes, Buildable, Context, Documented, Generic, Generics, Identified,
    Identifier, Input, Model, Output, Parameter, Parameterized, Treatment as TreatmentDescriptor,
    TreatmentBuildMode,
};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct Source {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    attributes: Attributes,
    models: HashMap<String, Arc<dyn Model>>,
    parameters: HashMap<String, Parameter>,
    outputs: HashMap<String, Output>,
    source_from: HashMap<String, Vec<String>>,
    auto_reference: Weak<Self>,
}

impl Source {
    pub fn new(
        identifier: Identifier,
        documentation: String,
        attributes: Attributes,
        models: Vec<(String, Arc<dyn Model>)>,
        parameters: Vec<Parameter>,
        source_from: Vec<(String, Vec<String>)>,
        outputs: Vec<Output>,
    ) -> Arc<Self> {
        #[cfg(not(feature = "doc"))]
        let _ = documentation;
        Arc::new_cyclic(|me| Self {
            identifier,
            #[cfg(feature = "doc")]
            documentation,
            attributes,
            models: HashMap::from_iter(models.into_iter().map(|(n, m)| (n.to_string(), m))),
            parameters: HashMap::from_iter(
                parameters.into_iter().map(|p| (p.name().to_string(), p)),
            ),
            outputs: HashMap::from_iter(outputs.into_iter().map(|o| (o.name().to_string(), o))),
            source_from: HashMap::from_iter(source_from.into_iter()),
            auto_reference: me.clone(),
        })
    }
}

impl Attribuable for Source {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl Identified for Source {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    fn make_use(&self, identifier: &Identifier) -> bool {
        self.models
            .iter()
            .any(|(_, model)| model.identifier() == identifier || model.make_use(identifier))
            || self.outputs.values().any(|output| {
                output
                    .described_type()
                    .final_type()
                    .data()
                    .map(|data| data.identifier() == identifier || data.make_use(identifier))
                    .unwrap_or(false)
            })
            || self.parameters.values().any(|parameter| {
                parameter
                    .described_type()
                    .final_type()
                    .data()
                    .map(|data| data.identifier() == identifier || data.make_use(identifier))
                    .unwrap_or(false)
            })
    }

    fn uses(&self) -> Vec<Identifier> {
        let mut uses = Vec::new();
        self.models.values().for_each(|model| {
            uses.push(model.identifier().clone());
            uses.extend(model.uses());
        });
        self.outputs.values().for_each(|output| {
            if let Some(data) = output.described_type().final_type().data() {
                uses.push(data.identifier().clone());
                uses.extend(data.uses());
            }
        });
        self.parameters.values().for_each(|parameter| {
            if let Some(data) = parameter.described_type().final_type().data() {
                uses.push(data.identifier().clone());
                uses.extend(data.uses());
            }
        });
        uses
    }
}

impl Documented for Source {
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

impl Parameterized for Source {
    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }

    fn as_identified(&self) -> Arc<dyn Identified> {
        self.auto_reference.upgrade().unwrap()
    }
}

impl Buildable<TreatmentBuildMode> for Source {
    fn build_mode(&self) -> TreatmentBuildMode {
        TreatmentBuildMode::Source(self.auto_reference.clone())
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "treatment {}", self.identifier.to_string())?;

        if !self.models.is_empty() {
            write!(
                f,
                "[{}]",
                self.models
                    .iter()
                    .map(|(n, m)| format!("{}: {}", n, m.identifier().to_string()))
                    .collect::<Vec<_>>()
                    .join(", "),
            )?;
        }

        write!(
            f,
            "({})",
            self.parameters()
                .iter()
                .map(|(_, p)| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )?;

        Ok(())
    }
}

impl TreatmentDescriptor for Source {
    fn inputs(&self) -> &HashMap<String, Input> {
        static HASHMAP: OnceCell<HashMap<String, Input>> = OnceCell::new();
        HASHMAP.get_or_init(|| HashMap::new())
    }

    fn outputs(&self) -> &HashMap<String, Output> {
        &self.outputs
    }

    fn models(&self) -> &HashMap<String, Arc<dyn Model>> {
        &self.models
    }

    fn contexts(&self) -> &HashMap<String, Arc<dyn Context>> {
        static HASHMAP: OnceCell<HashMap<String, Arc<dyn Context>>> = OnceCell::new();
        HASHMAP.get_or_init(|| HashMap::new())
    }

    fn source_from(&self) -> &HashMap<String, Vec<String>> {
        &self.source_from
    }

    fn as_identified(&self) -> Arc<dyn Identified> {
        self.auto_reference.upgrade().unwrap()
    }

    fn as_buildable(&self) -> Arc<dyn Buildable<TreatmentBuildMode>> {
        self.auto_reference.upgrade().unwrap()
    }

    fn as_parameterized(&self) -> Arc<dyn Parameterized> {
        self.auto_reference.upgrade().unwrap()
    }
}

impl Generics for Source {
    fn generics(&self) -> &Vec<Generic> {
        static VEC: OnceCell<Vec<Generic>> = OnceCell::new();
        VEC.get_or_init(|| Vec::new())
    }
}
