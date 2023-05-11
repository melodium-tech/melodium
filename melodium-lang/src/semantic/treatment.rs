//! Module dedicated to Treatment semantic analysis.

use super::common::Node;
use super::connection::Connection;
use super::declarative_element::{DeclarativeElement, DeclarativeElementType};
use super::declared_model::{DeclaredModel, RefersTo as DeclaredModelRefersTo};
use super::declared_parameter::DeclaredParameter;
use super::input::Input;
use super::model_instanciation::ModelInstanciation;
use super::output::Output;
use super::requirement::Requirement;
use super::script::Script;
use super::treatment_instanciation::TreatmentInstanciation;
use crate::error::ScriptError;
use crate::path::Path;
use crate::text::Treatment as TextTreatment;
use crate::ScriptResult;
use melodium_common::descriptor::{
    Collection, Entry, Identified, Identifier, Treatment as TreatmentTrait,
};
use melodium_engine::descriptor::Treatment as TreatmentDescriptor;
use melodium_engine::designer::Treatment as TreatmentDesigner;
use melodium_engine::LogicError;
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of a treatment.
///
/// It owns the whole [text treatment](TextTreatment).
#[derive(Debug)]
pub struct Treatment {
    pub text: TextTreatment,

    pub script: Weak<RwLock<Script>>,

    pub name: String,

    pub declared_models: Vec<Arc<RwLock<DeclaredModel>>>,
    pub parameters: Vec<Arc<RwLock<DeclaredParameter>>>,
    pub model_instanciations: Vec<Arc<RwLock<ModelInstanciation>>>,
    pub requirements: Vec<Arc<RwLock<Requirement>>>,
    pub inputs: Vec<Arc<RwLock<Input>>>,
    pub outputs: Vec<Arc<RwLock<Output>>>,
    pub treatment_instanciations: Vec<Arc<RwLock<TreatmentInstanciation>>>,
    pub connections: Vec<Arc<RwLock<Connection>>>,

    pub identifier: Option<Identifier>,
    pub descriptor: RwLock<Option<Arc<TreatmentDescriptor>>>,
}

impl Treatment {
    /// Create a new semantic treatment, based on textual treatment.
    ///
    /// * `script`: the parent script that "owns" this treatment.
    /// * `text`: the textual treatment.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    ///
    pub fn new(
        script: Arc<RwLock<Script>>,
        text: TextTreatment,
    ) -> ScriptResult<Arc<RwLock<Self>>> {
        let treatment = Arc::<RwLock<Self>>::new(RwLock::new(Self {
            text: text.clone(),
            script: Arc::downgrade(&script),
            name: text.name.string.clone(),
            declared_models: Vec::new(),
            parameters: Vec::new(),
            model_instanciations: Vec::new(),
            requirements: Vec::new(),
            inputs: Vec::new(),
            outputs: Vec::new(),
            treatment_instanciations: Vec::new(),
            connections: Vec::new(),
            identifier: None,
            descriptor: RwLock::new(None),
        }));
        let mut result = ScriptResult::new_success(Arc::clone(&treatment));

        {
            let borrowed_script = script.read().unwrap();

            let treatment = borrowed_script.find_treatment(&text.name.string);
            if treatment.is_some() {
                result = result.and_degrade_failure(ScriptResult::new_failure(
                    ScriptError::already_used_name(111, text.name.clone()),
                ));
            }

            let r#use = borrowed_script.find_use(&text.name.string);
            if r#use.is_some() {
                result = result.and_degrade_failure(ScriptResult::new_failure(
                    ScriptError::already_used_name(112, text.name),
                ));
            }
        }

        for c in text.configuration {
            if let Some(declared_model) =
                result.merge_degrade_failure(DeclaredModel::new(Arc::clone(&treatment), c))
            {
                treatment
                    .write()
                    .unwrap()
                    .declared_models
                    .push(declared_model);
            }
        }

        for p in text.parameters {
            if let Some(declared_parameter) = result.merge_degrade_failure(DeclaredParameter::new(
                Arc::clone(&treatment) as Arc<RwLock<dyn DeclarativeElement>>,
                p,
            )) {
                treatment
                    .write()
                    .unwrap()
                    .parameters
                    .push(declared_parameter);
            }
        }

        for m in text.models {
            if let Some(instancied_model) =
                result.merge_degrade_failure(ModelInstanciation::new(Arc::clone(&treatment), m))
            {
                treatment
                    .write()
                    .unwrap()
                    .model_instanciations
                    .push(Arc::clone(&instancied_model));
                if let Some(declared_model) = result
                    .merge_degrade_failure(DeclaredModel::from_instancied_model(instancied_model))
                {
                    treatment
                        .write()
                        .unwrap()
                        .declared_models
                        .push(declared_model);
                }
            }
        }

        for r in text.requirements {
            if let Some(requirement) =
                result.merge_degrade_failure(Requirement::new(Arc::clone(&treatment), r))
            {
                treatment.write().unwrap().requirements.push(requirement);
            }
        }

        for i in text.inputs {
            if let Some(input) = result.merge_degrade_failure(Input::new(Arc::clone(&treatment), i))
            {
                treatment.write().unwrap().inputs.push(input);
            }
        }

        for o in text.outputs {
            if let Some(output) =
                result.merge_degrade_failure(Output::new(Arc::clone(&treatment), o))
            {
                treatment.write().unwrap().outputs.push(output);
            }
        }

        for t in text.treatments {
            if let Some(treatment_instanciation) =
                result.merge_degrade_failure(TreatmentInstanciation::new(Arc::clone(&treatment), t))
            {
                treatment
                    .write()
                    .unwrap()
                    .treatment_instanciations
                    .push(treatment_instanciation);
            }
        }

        for c in text.connections {
            if let Some(connection) =
                result.merge_degrade_failure(Connection::new(Arc::clone(&treatment), c))
            {
                treatment.write().unwrap().connections.push(connection);
            }
        }

        result
    }

    /// Search for a declared model.
    pub fn find_declared_model(&self, name: &str) -> Option<&Arc<RwLock<DeclaredModel>>> {
        self.declared_models
            .iter()
            .find(|&m| m.read().unwrap().name == name)
    }

    /// Search for an instancied model.
    pub fn find_model_instanciation(&self, name: &str) -> Option<&Arc<RwLock<ModelInstanciation>>> {
        self.model_instanciations
            .iter()
            .find(|&m| m.read().unwrap().name == name)
    }

    /// Search for a requirement.
    pub fn find_requirement(&self, name: &str) -> Option<&Arc<RwLock<Requirement>>> {
        self.requirements
            .iter()
            .find(|&r| r.read().unwrap().name == name)
    }

    /// Search for an input.
    pub fn find_input(&self, name: &str) -> Option<&Arc<RwLock<Input>>> {
        self.inputs.iter().find(|&i| i.read().unwrap().name == name)
    }

    /// Search for an output.
    pub fn find_output(&self, name: &str) -> Option<&Arc<RwLock<Output>>> {
        self.outputs
            .iter()
            .find(|&o| o.read().unwrap().name == name)
    }

    /// Search for a treatment.
    pub fn find_treatment_instanciation(
        &self,
        name: &str,
    ) -> Option<&Arc<RwLock<TreatmentInstanciation>>> {
        self.treatment_instanciations
            .iter()
            .find(|&t| t.read().unwrap().name == name)
    }

    pub fn make_descriptor(&self, collection: &mut Collection) -> ScriptResult<()> {
        let mut result = ScriptResult::new_success(());
        let mut descriptor = TreatmentDescriptor::new(self.identifier.as_ref().unwrap().clone());

        if let Some(documentation) = &self.text.doc {
            descriptor.set_documentation(&documentation.string);
        }

        // We manage declaration of each model given to the treatment
        for rc_model in &self.declared_models {
            let borrowed_model = rc_model.read().unwrap();

            if borrowed_model.comes_from_instancied() {
                continue;
            }

            let model_identifier = match &borrowed_model.refers {
                DeclaredModelRefersTo::Use(u) => u
                    .reference
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
                DeclaredModelRefersTo::Model(m) => m
                    .reference
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
                _ => {
                    return ScriptResult::new_failure(ScriptError::reference_unset(
                        116,
                        format!("{:?}", &borrowed_model.refers),
                    ))
                }
            };

            if let Some(Entry::Model(model_descriptor)) = collection.get(&model_identifier) {
                descriptor.add_model(&borrowed_model.name, &model_descriptor)
            } else {
                result = result.and_degrade_failure(ScriptResult::new_failure(
                    LogicError::unexisting_model(
                        113,
                        descriptor.identifier().clone(),
                        model_identifier,
                        borrowed_model
                            .text
                            .as_ref()
                            .map(|text| text.name.into_ref()),
                    )
                    .into(),
                ));
            };
        }

        // We proceed to declaration of all other charateristics of the treatment

        for rc_parameter in &self.parameters {
            let borrowed_parameter = rc_parameter.read().unwrap();
            if let Some(parameter_descriptor) =
                result.merge_degrade_failure(borrowed_parameter.make_descriptor())
            {
                descriptor.add_parameter(parameter_descriptor);
            }
        }

        for rc_input in &self.inputs {
            let borrowed_input = rc_input.read().unwrap();
            if let Some(input_descriptor) =
                result.merge_degrade_failure(borrowed_input.make_descriptor())
            {
                descriptor.add_input(input_descriptor);
            }
        }

        for rc_output in &self.outputs {
            let borrowed_output = rc_output.read().unwrap();
            if let Some(output_descriptor) =
                result.merge_degrade_failure(borrowed_output.make_descriptor())
            {
                descriptor.add_output(output_descriptor);
            }
        }

        for rc_requirement in &self.requirements {
            let borrowed_requirement = rc_requirement.read().unwrap();

            if let Some(Entry::Context(context)) =
                collection.get(borrowed_requirement.type_identifier.as_ref().unwrap())
            {
                descriptor.add_context(context);
            } else {
                result = result.and_degrade_failure(ScriptResult::new_failure(
                    LogicError::unexisting_context(
                        114,
                        descriptor.identifier().clone(),
                        borrowed_requirement
                            .type_identifier
                            .as_ref()
                            .unwrap()
                            .clone(),
                        None,
                    )
                    .into(),
                ));
            }
        }

        if result.is_success() {
            let descriptor = descriptor.commit();

            collection.insert(Entry::Treatment(
                Arc::clone(&descriptor) as Arc<dyn TreatmentTrait>
            ));

            *self.descriptor.write().unwrap() = Some(descriptor);
        }

        result
    }

    pub fn make_design(&self, collection: &Arc<Collection>) -> ScriptResult<()> {
        let borrowed_descriptor = self.descriptor.read().unwrap();
        let descriptor = if let Some(descriptor) = &*borrowed_descriptor {
            descriptor
        } else {
            return ScriptResult::new_failure(ScriptError::no_descriptor(
                115,
                self.text.name.clone(),
            ));
        };

        let mut result = ScriptResult::new_success(());

        let rc_designer: Arc<RwLock<TreatmentDesigner>> = if let Some(designer) = result
            .merge_degrade_failure(ScriptResult::from(
                descriptor.designer(Some(self.text.name.into_ref())),
            )) {
            designer
        } else {
            return result;
        };
        rc_designer
            .write()
            .unwrap()
            .set_collection(Arc::clone(collection));

        // Models instanciations
        for rc_instancied_model in &self.model_instanciations {
            let instancied_model = rc_instancied_model.read().unwrap();

            let tmp_status = rc_designer.write().unwrap().add_model_instanciation(
                instancied_model.type_identifier.as_ref().unwrap(),
                &instancied_model.name,
                Some(instancied_model.text.name.into_ref()),
            );
            if let Some(instanciation_designer) =
                result.merge_degrade_failure(ScriptResult::from(tmp_status))
            {
                result = result
                    .and_degrade_failure(instancied_model.make_design(&instanciation_designer));
            }
        }

        // Treatments
        for rc_treatment in &self.treatment_instanciations {
            let treatment = rc_treatment.read().unwrap();

            let tmp_status = rc_designer.write().unwrap().add_treatment(
                treatment.type_identifier.as_ref().unwrap(),
                &treatment.name,
                Some(treatment.text.name.into_ref()),
            );
            if let Some(treatment_designer) =
                result.merge_degrade_failure(ScriptResult::from(tmp_status))
            {
                result = result.and_degrade_failure(treatment.make_design(&treatment_designer));
            }
        }

        // Connections
        for rc_connection in &self.connections {
            let connection = rc_connection.read().unwrap();

            result = result
                .and_degrade_failure(connection.make_design(&mut rc_designer.write().unwrap()));
        }

        result = result.and_degrade_failure(ScriptResult::from(descriptor.commit_design()));

        result
    }
}

impl Node for Treatment {
    fn make_references(&mut self, path: &Path) -> ScriptResult<()> {
        self.identifier = path.to_identifier(&self.name);

        ScriptResult::new_success(())
    }

    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {
        let mut children: Vec<Arc<RwLock<dyn Node>>> = Vec::new();

        self.declared_models
            .iter()
            .for_each(|m| children.push(Arc::clone(&m) as Arc<RwLock<dyn Node>>));
        self.parameters
            .iter()
            .for_each(|p| children.push(Arc::clone(&p) as Arc<RwLock<dyn Node>>));
        self.model_instanciations
            .iter()
            .for_each(|m| children.push(Arc::clone(&m) as Arc<RwLock<dyn Node>>));
        self.requirements
            .iter()
            .for_each(|r| children.push(Arc::clone(&r) as Arc<RwLock<dyn Node>>));
        self.inputs
            .iter()
            .for_each(|i| children.push(Arc::clone(&i) as Arc<RwLock<dyn Node>>));
        self.outputs
            .iter()
            .for_each(|o| children.push(Arc::clone(&o) as Arc<RwLock<dyn Node>>));
        self.treatment_instanciations
            .iter()
            .for_each(|t| children.push(Arc::clone(&t) as Arc<RwLock<dyn Node>>));
        self.connections
            .iter()
            .for_each(|c| children.push(Arc::clone(&c) as Arc<RwLock<dyn Node>>));

        children
    }
}

impl DeclarativeElement for Treatment {
    fn declarative_element(&self) -> DeclarativeElementType {
        DeclarativeElementType::Treatment(&self)
    }

    /// Search for a parameter.
    fn find_declared_parameter(&self, name: &str) -> Option<&Arc<RwLock<DeclaredParameter>>> {
        self.parameters
            .iter()
            .find(|&p| p.read().unwrap().name == name)
    }
}
