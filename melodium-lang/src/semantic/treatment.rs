//! Module dedicated to Treatment semantic analysis.

use super::common::Node;

use crate::error::{wrap_logic_error, ScriptError};
use crate::path::Path;
use crate::text::Treatment as TextTreatment;
use melodium_common::descriptor::{Collection, Entry, Identifier, Treatment as TreatmentTrait};
use melodium_engine::descriptor::Treatment as TreatmentDescriptor;
use std::sync::{Arc, RwLock, Weak};

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
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](../common/trait.Node.html).
    ///
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script as TextScript;
    /// # use melodium::script::semantic::script::Script;
    /// let address = "melodium-tests/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    ///
    /// let text_script = TextScript::build(&raw_text)?;
    ///
    /// let script = Script::new(text_script)?;
    /// // Internally, Script::new call Treatment::new(Arc::clone(&script), text_treatment)
    ///
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_treatment = borrowed_script.find_treatment("AudioToHpcpImage").unwrap().read().unwrap();
    ///
    /// assert_eq!(borrowed_treatment.declared_models.len(), 1);
    /// assert_eq!(borrowed_treatment.parameters.len(), 3);
    /// assert_eq!(borrowed_treatment.requirements.len(), 2);
    /// assert_eq!(borrowed_treatment.treatments.len(), 4);
    /// assert!(borrowed_treatment.origin.is_some());
    /// assert_eq!(borrowed_treatment.origin.as_ref().unwrap().read().unwrap().name, "AudioSignal");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(
        script: Arc<RwLock<Script>>,
        text: TextTreatment,
    ) -> Result<Arc<RwLock<Self>>, ScriptError> {
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

        {
            let borrowed_script = script.read().unwrap();

            let treatment = borrowed_script.find_treatment(&text.name.string);
            if treatment.is_some() {
                return Err(ScriptError::semantic(
                    "Treatment '".to_string() + &text.name.string + "' is already declared.",
                    text.name.position,
                ));
            }

            let r#use = borrowed_script.find_use(&text.name.string);
            if r#use.is_some() {
                return Err(ScriptError::semantic(
                    "Element '".to_string() + &text.name.string + "' is already declared as used.",
                    text.name.position,
                ));
            }
        }

        for c in text.configuration {
            let declared_model = DeclaredModel::new(Arc::clone(&treatment), c)?;
            treatment
                .write()
                .unwrap()
                .declared_models
                .push(declared_model);
        }

        for p in text.parameters {
            let declared_parameter = DeclaredParameter::new(
                Arc::clone(&treatment) as Arc<RwLock<dyn DeclarativeElement>>,
                p,
            )?;
            treatment
                .write()
                .unwrap()
                .parameters
                .push(declared_parameter);
        }

        for m in text.models {
            let instancied_model = ModelInstanciation::new(Arc::clone(&treatment), m)?;
            treatment
                .write()
                .unwrap()
                .model_instanciations
                .push(Arc::clone(&instancied_model));
            let declared_model = DeclaredModel::from_instancied_model(instancied_model)?;
            treatment
                .write()
                .unwrap()
                .declared_models
                .push(declared_model);
        }

        for r in text.requirements {
            let requirement = Requirement::new(Arc::clone(&treatment), r)?;
            treatment.write().unwrap().requirements.push(requirement);
        }

        for i in text.inputs {
            let input = Input::new(Arc::clone(&treatment), i)?;
            treatment.write().unwrap().inputs.push(input);
        }

        for o in text.outputs {
            let output = Output::new(Arc::clone(&treatment), o)?;
            treatment.write().unwrap().outputs.push(output);
        }

        for t in text.treatments {
            let treatment_instanciation = TreatmentInstanciation::new(Arc::clone(&treatment), t)?;
            treatment
                .write()
                .unwrap()
                .treatment_instanciations
                .push(treatment_instanciation);
        }

        for c in text.connections {
            let connection = Connection::new(Arc::clone(&treatment), c)?;
            treatment.write().unwrap().connections.push(connection);
        }

        Ok(treatment)
    }

    /// Search for a declared model.
    ///
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script as TextScript;
    /// # use melodium::script::semantic::script::Script;
    /// let address = "melodium-tests/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    ///
    /// let text_script = TextScript::build(&raw_text)?;
    ///
    /// let script = Script::new(text_script)?;
    ///
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_treatment = borrowed_script.find_treatment("AudioToHpcpImage").unwrap().read().unwrap();
    ///
    /// let audio_manager = borrowed_treatment.find_declared_model("AudioManager");
    /// let dont_exist = borrowed_treatment.find_declared_model("DontExist");
    /// assert!(audio_manager.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_declared_model(&self, name: &str) -> Option<&Arc<RwLock<DeclaredModel>>> {
        self.declared_models
            .iter()
            .find(|&m| m.read().unwrap().name == name)
    }

    /// Search for an instancied model.
    ///
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script as TextScript;
    /// # use melodium::script::semantic::script::Script;
    /// let address = "melodium-tests/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    ///
    /// let text_script = TextScript::build(&raw_text)?;
    ///
    /// let script = Script::new(text_script)?;
    ///
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_treatment = borrowed_script.find_treatment("Main").unwrap().read().unwrap();
    ///
    /// let audio = borrowed_treatment.find_instancied_model("Audio");
    /// let dont_exist = borrowed_treatment.find_instancied_model("DontExist");
    /// assert!(audio.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_model_instanciation(&self, name: &str) -> Option<&Arc<RwLock<ModelInstanciation>>> {
        self.model_instanciations
            .iter()
            .find(|&m| m.read().unwrap().name == name)
    }

    /// Search for a requirement.
    ///
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script as TextScript;
    /// # use melodium::script::semantic::script::Script;
    /// let address = "melodium-tests/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    ///
    /// let text_script = TextScript::build(&raw_text)?;
    ///
    /// let script = Script::new(text_script)?;
    ///
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_treatment = borrowed_script.find_treatment("AudioToHpcpImage").unwrap().read().unwrap();
    ///
    /// let signal = borrowed_treatment.find_requirement("@Signal");
    /// let dont_exist = borrowed_treatment.find_requirement("@DontExist");
    /// assert!(signal.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_requirement(&self, name: &str) -> Option<&Arc<RwLock<Requirement>>> {
        self.requirements
            .iter()
            .find(|&r| r.read().unwrap().name == name)
    }

    /// Search for an input.
    ///
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script as TextScript;
    /// # use melodium::script::semantic::script::Script;
    /// let address = "melodium-tests/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    ///
    /// let text_script = TextScript::build(&raw_text)?;
    ///
    /// let script = Script::new(text_script)?;
    ///
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_treatment = borrowed_script.find_treatment("HPCP").unwrap().read().unwrap();
    ///
    /// let spectrum = borrowed_treatment.find_input("spectrum");
    /// let dont_exist = borrowed_treatment.find_input("dontExist");
    /// assert!(spectrum.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_input(&self, name: &str) -> Option<&Arc<RwLock<Input>>> {
        self.inputs.iter().find(|&i| i.read().unwrap().name == name)
    }

    /// Search for an output.
    ///
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script as TextScript;
    /// # use melodium::script::semantic::script::Script;
    /// let address = "melodium-tests/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    ///
    /// let text_script = TextScript::build(&raw_text)?;
    ///
    /// let script = Script::new(text_script)?;
    ///
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_treatment = borrowed_script.find_treatment("HPCP").unwrap().read().unwrap();
    ///
    /// let hpcp = borrowed_treatment.find_output("hpcp");
    /// let dont_exist = borrowed_treatment.find_output("dontExist");
    /// assert!(hpcp.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_output(&self, name: &str) -> Option<&Arc<RwLock<Output>>> {
        self.outputs
            .iter()
            .find(|&o| o.read().unwrap().name == name)
    }

    /// Search for a treatment.
    ///
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script as TextScript;
    /// # use melodium::script::semantic::script::Script;
    /// let address = "melodium-tests/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    ///
    /// let text_script = TextScript::build(&raw_text)?;
    ///
    /// let script = Script::new(text_script)?;
    ///
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_treatment = borrowed_script.find_treatment("Spectrum").unwrap().read().unwrap();
    ///
    /// let core_frame_cutter = borrowed_treatment.find_treatment("CoreFrameCutter");
    /// let dont_exist = borrowed_treatment.find_treatment("DontExist");
    /// assert!(core_frame_cutter.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_treatment_instanciation(
        &self,
        name: &str,
    ) -> Option<&Arc<RwLock<TreatmentInstanciation>>> {
        self.treatment_instanciations
            .iter()
            .find(|&t| t.read().unwrap().name == name)
    }

    pub fn make_descriptor(&self, collection: &mut Collection) -> Result<(), ScriptError> {
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
                _ => panic!("{:?}", &borrowed_model.refers),
            };

            if let Some(Entry::Model(model_descriptor)) = collection.get(&model_identifier) {
                descriptor.add_model(&borrowed_model.name, &model_descriptor)
            } else {
                // Only the RefersTo::Use case can exist at that point.
                return Err(ScriptError::semantic(
                    "Model \"".to_string() + &model_identifier.to_string() + "\" does not exist.",
                    borrowed_model.text.as_ref().unwrap().name.position,
                ));
            };
        }

        // We proceed to declaration of all other charateristics of the treatment

        for rc_parameter in &self.parameters {
            let borrowed_parameter = rc_parameter.read().unwrap();
            let parameter_descriptor = borrowed_parameter.make_descriptor()?;

            descriptor.add_parameter(parameter_descriptor);
        }

        for rc_input in &self.inputs {
            let borrowed_input = rc_input.read().unwrap();
            let input_descriptor = borrowed_input.make_descriptor()?;

            descriptor.add_input(input_descriptor);
        }

        for rc_output in &self.outputs {
            let borrowed_output = rc_output.read().unwrap();
            let output_descriptor = borrowed_output.make_descriptor()?;

            descriptor.add_output(output_descriptor);
        }

        for rc_requirement in &self.requirements {
            let borrowed_requirement = rc_requirement.read().unwrap();

            let context =
                match collection.get(borrowed_requirement.type_identifier.as_ref().unwrap()) {
                    Some(Entry::Context(c)) => Ok(c),
                    _ => Err(ScriptError::semantic(
                        format!(
                            r#"Context "{}" does not exist."#,
                            borrowed_requirement.text.name.string
                        ),
                        borrowed_requirement.text.name.position,
                    )),
                }?;

            descriptor.add_context(context);
        }

        let descriptor = descriptor.commit();

        collection.insert(Entry::Treatment(
            Arc::clone(&descriptor) as Arc<dyn TreatmentTrait>
        ));

        *self.descriptor.write().unwrap() = Some(descriptor);

        Ok(())
    }

    pub fn make_design(&self, collection: &Arc<Collection>) -> Result<(), ScriptError> {
        let borrowed_descriptor = self.descriptor.read().unwrap();
        let descriptor = if let Some(descriptor) = &*borrowed_descriptor {
            descriptor
        } else {
            return Err(ScriptError::no_descriptor());
        };

        let rc_designer = descriptor.designer()?;
        rc_designer
            .write()
            .unwrap()
            .set_collection(Arc::clone(collection));

        // Models instanciations
        for rc_instancied_model in &self.model_instanciations {
            let instancied_model = rc_instancied_model.read().unwrap();

            let instanciation_designer = wrap_logic_error!(
                rc_designer.write().unwrap().add_model_instanciation(
                    instancied_model.type_identifier.as_ref().unwrap(),
                    &instancied_model.name
                ),
                instancied_model.text.name.position
            );

            instancied_model.make_design(&instanciation_designer)?;
        }

        // Treatments
        for rc_treatment in &self.treatment_instanciations {
            let treatment = rc_treatment.read().unwrap();

            let treatment_designer = wrap_logic_error!(
                rc_designer
                    .write()
                    .unwrap()
                    .add_treatment(treatment.type_identifier.as_ref().unwrap(), &treatment.name),
                treatment.text.name.position
            );

            treatment.make_design(&treatment_designer)?;
        }

        // Connections
        for rc_connection in &self.connections {
            let connection = rc_connection.read().unwrap();

            connection.make_design(&mut rc_designer.write().unwrap())?;
        }

        wrap_logic_error!(descriptor.commit_design(), self.text.name.position);

        Ok(())
    }
}

impl Node for Treatment {
    fn make_references(&mut self, path: &Path) -> Result<(), ScriptError> {
        self.identifier = path.to_identifier(&self.name);

        Ok(())
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
    ///
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::script::Script as TextScript;
    /// # use melodium::script::semantic::script::Script;
    /// # use melodium::script::semantic::declarative_element::DeclarativeElement;
    /// let address = "melodium-tests/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    ///
    /// let text_script = TextScript::build(&raw_text)?;
    ///
    /// let script = Script::new(text_script)?;
    ///
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_treatment = borrowed_script.find_treatment("Spectrum").unwrap().read().unwrap();
    ///
    /// let frame_size = borrowed_treatment.find_declared_parameter("frameSize");
    /// let dont_exist = borrowed_treatment.find_declared_parameter("dontExist");
    /// assert!(frame_size.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    fn find_declared_parameter(&self, name: &str) -> Option<&Arc<RwLock<DeclaredParameter>>> {
        self.parameters
            .iter()
            .find(|&p| p.read().unwrap().name == name)
    }
}
