use crate::design::Treatment as Design;
use crate::designer::{Reference, Treatment as Designer};
use crate::error::{LogicError, LogicResult};
use core::fmt::{Display, Formatter, Result as FmtResult};
use melodium_common::descriptor::{
    Attribuable, Attribute, Attributes, Buildable, Collection, Context, Documented, Entry, Generic,
    Generics, Identified, Identifier, Input, Model, Output, Parameter, Parameterized, Status,
    Treatment as TreatmentDescriptor, TreatmentBuildMode,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock, Weak};

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
    contexts: HashMap<String, Arc<dyn Context>>,
    designer: Mutex<Option<Arc<RwLock<Designer>>>>,
    design: Mutex<Option<Arc<Design>>>,
    auto_reference: Weak<Self>,
}

impl Treatment {
    pub fn new(identifier: Identifier) -> Self {
        Self {
            identifier,
            #[cfg(feature = "doc")]
            documentation: String::new(),
            attributes: Attributes::default(),
            generics: Vec::new(),
            models: HashMap::new(),
            parameters: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            contexts: HashMap::new(),
            designer: Mutex::new(None),
            design: Mutex::new(None),
            auto_reference: Weak::default(),
        }
    }

    pub fn set_identifier(&mut self, identifier: Identifier) {
        self.identifier = identifier;
    }

    pub fn reset_designer(&self) {
        let mut option_designer = self.designer.lock().expect("Mutex poisoned");
        *option_designer = None;
    }

    pub fn designer(
        &self,
        collection: Arc<Collection>,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> LogicResult<Arc<RwLock<Designer>>> {
        if self.auto_reference.strong_count() == 0 {
            return Status::new_failure(
                LogicError::uncommited_descriptor(3, self.identifier.clone(), None).into(),
            );
        }

        let mut option_designer = self.designer.lock().expect("Mutex poisoned");

        if let Some(designer_ref) = &*option_designer {
            Status::new_success(designer_ref.clone())
        } else {
            let new_designer = Designer::new(
                &self.auto_reference.upgrade().unwrap(),
                collection,
                design_reference,
            );

            *option_designer = Some(new_designer.clone());

            Status::new_success(new_designer)
        }
    }

    pub fn commit_design(&self) -> LogicResult<()> {
        let option_designer = self.designer.lock().expect("Mutex poisoned");
        let mut option_design = self.design.lock().expect("Mutex poisoned");

        if let Some(designer_ref) = &*option_designer {
            let designer = designer_ref.read().unwrap();
            let mut result_design = designer.design();

            if result_design.is_success() && result_design.has_errors() {
                result_design =
                    result_design.and(Status::new_failure(LogicError::erroneous_design(
                        68,
                        self.identifier.clone(),
                        designer.design_reference().clone(),
                    )));
            }

            result_design.and_then(|design| {
                *option_design = Some(Arc::new(design));
                Status::new_success(())
            })
        } else {
            Status::new_failure(LogicError::no_designer(67, self.identifier.clone(), None))
        }
    }

    pub fn design(&self) -> LogicResult<Arc<Design>> {
        let option_design = self.design.lock().expect("Mutex poisoned");

        option_design
            .as_ref()
            .map(|design| Arc::clone(design))
            .ok_or_else(|| LogicError::unavailable_design(5, self.identifier.clone(), None).into())
            .into()
    }

    pub fn update_with_collection(
        &mut self,
        collection: &Collection,
        replace: &HashMap<Identifier, Identifier>,
    ) -> LogicResult<()> {
        let mut result = LogicResult::new_success(());

        let mut new_models = HashMap::new();
        for (name, model) in &self.models {
            let model_identifier = replace
                .get(model.identifier())
                .unwrap_or_else(|| model.identifier());
            if let Some(Entry::Model(model)) = collection.get(&model_identifier.into()) {
                new_models.insert(name.clone(), model.clone());
            } else {
                result.errors_mut().push(LogicError::unexisting_model(
                    206,
                    self.identifier.clone(),
                    model_identifier.into(),
                    None,
                ))
            }
        }
        self.models = new_models;

        let mut new_contexts = HashMap::new();
        for (name, context) in &self.contexts {
            let context_identifier = replace
                .get(context.identifier())
                .unwrap_or_else(|| context.identifier());
            if let Some(Entry::Context(context)) = collection.get(&context_identifier.into()) {
                new_contexts.insert(name.clone(), context.clone());
            } else {
                result.errors_mut().push(LogicError::unexisting_context(
                    207,
                    self.identifier.clone(),
                    context_identifier.into(),
                    None,
                ))
            }
        }
        self.contexts = new_contexts;

        result
    }

    pub fn set_documentation(&mut self, documentation: &str) {
        #[cfg(feature = "doc")]
        {
            self.documentation = String::from(documentation);
        }
        #[cfg(not(feature = "doc"))]
        let _ = documentation;
    }

    pub fn add_attribute(&mut self, name: String, attribute: Attribute) {
        self.attributes.insert(name, attribute);
    }

    pub fn remove_attribute(&mut self, name: &str) -> bool {
        match self.attributes.remove(name) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn add_generic(&mut self, generic: Generic) {
        self.generics.retain(|gen| gen.name != generic.name);
        self.generics.push(generic);
    }

    pub fn remove_generic(&mut self, name: &str) -> bool {
        let mut found = false;
        self.generics.retain(|gen| {
            if gen.name != name {
                found = true;
                false
            } else {
                true
            }
        });
        found
    }

    pub fn add_model(&mut self, name: &str, model: &Arc<dyn Model>) {
        self.models.insert(name.to_string(), Arc::clone(model));
    }

    pub fn remove_model(&mut self, name: &str) -> bool {
        self.models.remove(name).is_some()
    }

    pub fn add_parameter(&mut self, parameter: Parameter) {
        self.parameters
            .insert(parameter.name().to_string(), parameter);
    }

    pub fn remove_parameter(&mut self, name: &str) -> bool {
        self.parameters.remove(name).is_some()
    }

    pub fn add_input(&mut self, input: Input) {
        self.inputs.insert(input.name().to_string(), input);
    }

    pub fn remove_input(&mut self, name: &str) -> bool {
        match self.inputs.remove(name) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn add_output(&mut self, output: Output) {
        self.outputs.insert(output.name().to_string(), output);
    }

    pub fn remove_output(&mut self, name: &str) -> bool {
        match self.outputs.remove(name) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn add_context(&mut self, context: &Arc<dyn Context>) {
        self.contexts
            .insert(context.name().to_string(), context.clone());
    }

    pub fn remove_context(&mut self, name: &str) -> bool {
        match self.contexts.remove(name) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn commit(self) -> Arc<Self> {
        Arc::new_cyclic(|me| Self {
            identifier: self.identifier,
            #[cfg(feature = "doc")]
            documentation: self.documentation,
            attributes: self.attributes,
            generics: self.generics,
            models: self.models,
            parameters: self.parameters,
            inputs: self.inputs,
            outputs: self.outputs,
            contexts: self.contexts,
            designer: self.designer,
            design: self.design,
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
            || self
                .contexts
                .values()
                .any(|context| context.identifier() == identifier || context.make_use(identifier))
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
            || self
                .design
                .lock()
                .unwrap()
                .as_ref()
                .map(|design| design.make_use(identifier))
                .unwrap_or(false)
            || self
                .designer
                .lock()
                .unwrap()
                .as_ref()
                .map(|designer| designer.read().unwrap().make_use(identifier))
                .unwrap_or(false)
    }

    fn uses(&self) -> Vec<Identifier> {
        let mut uses = Vec::new();
        self.models.values().for_each(|model| {
            uses.push(model.identifier().clone());
            uses.extend(model.uses());
        });
        self.contexts.values().for_each(|context| {
            uses.push(context.identifier().clone());
            uses.extend(context.uses());
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
        if let Some(design) = self.design.lock().unwrap().as_ref() {
            uses.extend(design.uses());
        }
        if let Some(designer) = self.designer.lock().unwrap().as_ref() {
            uses.extend(designer.read().unwrap().uses());
        }
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
        TreatmentBuildMode::Designed()
    }
}

impl Clone for Treatment {
    /**
     * Clone treatment descriptor.
     *
     * The descriptor and its inner descriptive elements are all cloned, but not the designer nor the related design.
     * The cloned descriptor need to be commited.
     */
    fn clone(&self) -> Self {
        Self {
            identifier: self.identifier.clone(),
            #[cfg(feature = "doc")]
            documentation: self.documentation.clone(),
            attributes: self.attributes.clone(),
            generics: self.generics.clone(),
            models: self.models.clone(),
            parameters: self.parameters.clone(),
            inputs: self.inputs.clone(),
            outputs: self.outputs.clone(),
            contexts: self.contexts.clone(),
            designer: Mutex::new(None),
            design: Mutex::new(None),
            auto_reference: Weak::default(),
        }
    }
}

impl Display for Treatment {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
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
        &self.contexts
    }

    fn source_from(&self) -> &HashMap<String, Vec<String>> {
        lazy_static! {
            static ref HASHMAP: HashMap<String, Vec<String>> = HashMap::new();
        };
        &HASHMAP
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
