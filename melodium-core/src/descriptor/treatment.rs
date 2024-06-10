use core::fmt::{Display, Formatter, Result};
use melodium_common::descriptor::{
    Attribuable, Attributes, Buildable, Context, Documented, Generic, Generics, Identified,
    Identifier, Input, Model, Output, Parameter, Parameterized, Treatment as TreatmentDescriptor,
    TreatmentBuildMode,
};
use melodium_common::executive::Treatment as ExecutiveTreatment;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct Treatment {
    identifier: Identifier,
    #[cfg(feature = "doc")]
    documentation: String,
    attributes: Attributes,
    generics: Vec<Generic>,
    models: HashMap<String, Arc<dyn Model>>,
    parameters: HashMap<String, Parameter>,
    inputs: HashMap<String, Input>,
    outputs: HashMap<String, Output>,
    source_from: HashMap<String, Vec<String>>,
    build_fn: fn() -> Arc<dyn ExecutiveTreatment>,
    auto_reference: Weak<Self>,
}

impl Treatment {
    pub fn new(
        identifier: Identifier,
        documentation: String,
        attributes: Attributes,
        generics: Vec<Generic>,
        models: Vec<(String, Arc<dyn Model>)>,
        source_from: Vec<(String, Vec<String>)>,
        parameters: Vec<Parameter>,
        inputs: Vec<Input>,
        outputs: Vec<Output>,
        build_fn: fn() -> Arc<dyn ExecutiveTreatment>,
    ) -> Arc<Self> {
        #[cfg(not(feature = "doc"))]
        let _ = documentation;
        Arc::new_cyclic(|me| Self {
            identifier,
            #[cfg(feature = "doc")]
            documentation,
            attributes,
            generics,
            models: HashMap::from_iter(models.into_iter().map(|(n, m)| (n, m))),
            parameters: HashMap::from_iter(
                parameters.into_iter().map(|p| (p.name().to_string(), p)),
            ),
            inputs: HashMap::from_iter(inputs.into_iter().map(|i| (i.name().to_string(), i))),
            outputs: HashMap::from_iter(outputs.into_iter().map(|o| (o.name().to_string(), o))),
            source_from: HashMap::from_iter(source_from.into_iter()),
            build_fn,
            auto_reference: me.clone(),
        })
    }
}

impl Attribuable for Treatment {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl Identified for Treatment {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }

    fn make_use(&self, identifier: &Identifier) -> bool {
        self.models
            .iter()
            .any(|(_, model)| model.identifier() == identifier || model.make_use(identifier))
            || self.inputs.values().any(|input| {
                input
                    .described_type()
                    .final_type()
                    .data()
                    .map(|data| data.identifier() == identifier || data.make_use(identifier))
                    .unwrap_or(false)
            })
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
        self.inputs.values().for_each(|input| {
            if let Some(data) = input.described_type().final_type().data() {
                uses.push(data.identifier().clone());
                uses.extend(data.uses());
            }
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

impl Documented for Treatment {
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

impl Parameterized for Treatment {
    fn parameters(&self) -> &HashMap<String, Parameter> {
        &self.parameters
    }

    fn as_identified(&self) -> Arc<dyn Identified> {
        self.auto_reference.upgrade().unwrap()
    }
}

impl Buildable<TreatmentBuildMode> for Treatment {
    fn build_mode(&self) -> TreatmentBuildMode {
        TreatmentBuildMode::Compiled(self.build_fn, self.auto_reference.clone())
    }
}

impl Display for Treatment {
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

impl TreatmentDescriptor for Treatment {
    fn inputs(&self) -> &HashMap<String, Input> {
        &self.inputs
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

impl Generics for Treatment {
    fn generics(&self) -> &Vec<Generic> {
        &self.generics
    }
}
