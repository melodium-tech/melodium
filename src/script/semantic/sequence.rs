
//! Module dedicated to Sequence semantic analysis.

use super::common::Node;

use std::sync::{Arc, Weak, RwLock};
use crate::script::error::ScriptError;
use crate::script::text::Sequence as TextSequence;
use crate::script::path::Path;
use crate::logic::descriptor::identifier::Identifier;

use super::script::Script;
use super::declared_model::DeclaredModel;
use super::declarative_element::{DeclarativeElement, DeclarativeElementType};
use super::declared_parameter::DeclaredParameter;
use super::instancied_model::InstanciedModel;
use super::requirement::Requirement;
use super::input::Input;
use super::output::Output;
use super::treatment::Treatment;
use super::connection::Connection;

/// Structure managing and describing semantic of a sequence.
/// 
/// It owns the whole [text sequence](../../text/sequence/struct.Sequence.html).
pub struct Sequence {
    pub text: TextSequence,

    pub script: Weak<RwLock<Script>>,

    pub name: String,

    pub declared_models: Vec<Arc<RwLock<DeclaredModel>>>,
    pub parameters: Vec<Arc<RwLock<DeclaredParameter>>>,
    pub instancied_models: Vec<Arc<RwLock<InstanciedModel>>>,
    pub requirements: Vec<Arc<RwLock<Requirement>>>,
    pub origin: Option<Arc<RwLock<Treatment>>>,
    pub inputs: Vec<Arc<RwLock<Input>>>,
    pub outputs: Vec<Arc<RwLock<Output>>>,
    pub treatments: Vec<Arc<RwLock<Treatment>>>,
    pub connections: Vec<Arc<RwLock<Connection>>>,

    pub identifier: Option<Identifier>,
}

impl Sequence {
    /// Create a new semantic sequence, based on textual sequence.
    /// 
    /// * `script`: the parent script that "owns" this sequence.
    /// * `text`: the textual sequence.
    /// 
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](../common/trait.Node.html).
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// // Internally, Script::new call Sequence::new(Arc::clone(&script), text_sequence)
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("AudioToHpcpImage").unwrap().read().unwrap();
    /// 
    /// assert_eq!(borrowed_sequence.declared_models.len(), 1);
    /// assert_eq!(borrowed_sequence.parameters.len(), 3);
    /// assert_eq!(borrowed_sequence.requirements.len(), 2);
    /// assert_eq!(borrowed_sequence.treatments.len(), 4);
    /// assert!(borrowed_sequence.origin.is_some());
    /// assert_eq!(borrowed_sequence.origin.as_ref().unwrap().read().unwrap().name, "AudioSignal");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(script: Arc<RwLock<Script>>, text: TextSequence) -> Result<Arc<RwLock<Self>>, ScriptError> {

        let sequence = Arc::<RwLock<Self>>::new(RwLock::new(Self {
            text: text.clone(),
            script: Arc::downgrade(&script),
            name: text.name.string.clone(),
            declared_models: Vec::new(),
            parameters: Vec::new(),
            instancied_models: Vec::new(),
            requirements: Vec::new(),
            origin: None,
            inputs: Vec::new(),
            outputs: Vec::new(),
            treatments: Vec::new(),
            connections: Vec::new(),
            identifier: None,
        }));

        {
            let borrowed_script = script.read().unwrap();

            let sequence = borrowed_script.find_sequence(&text.name.string);
            if sequence.is_some() {
                return Err(ScriptError::semantic("Sequence '".to_string() + &text.name.string + "' is already declared.", text.name.position))
            }

            let r#use = borrowed_script.find_use(&text.name.string);
            if r#use.is_some() {
                return Err(ScriptError::semantic("Element '".to_string() + &text.name.string + "' is already declared as used.", text.name.position))
            }
        }

        for c in text.configuration {
            let declared_model = DeclaredModel::new(Arc::clone(&sequence), c)?;
            sequence.write().unwrap().declared_models.push(declared_model);
        }

        for p in text.parameters {
            let declared_parameter = DeclaredParameter::new(Arc::clone(&sequence) as Arc<RwLock<dyn DeclarativeElement>>, p)?;
            sequence.write().unwrap().parameters.push(declared_parameter);
        }

        for m in text.models {
            let instancied_model = InstanciedModel::new(Arc::clone(&sequence), m)?;
            sequence.write().unwrap().instancied_models.push(Arc::clone(&instancied_model));
            let declared_model = DeclaredModel::from_instancied_model(instancied_model)?;
            sequence.write().unwrap().declared_models.push(declared_model);
        }

        for r in text.requirements {
            let requirement = Requirement::new(Arc::clone(&sequence), r)?;
            sequence.write().unwrap().requirements.push(requirement);
        }

        if text.origin.is_some() {

            let origin = Treatment::new(Arc::clone(&sequence), text.origin.unwrap())?;

            let mut borrowed_sequence = sequence.write().unwrap();
            borrowed_sequence.origin = Some(Arc::clone(&origin));
            borrowed_sequence.treatments.push(Arc::clone(&origin));
        }

        for i in text.inputs {
            let input = Input::new(Arc::clone(&sequence), i)?;
            sequence.write().unwrap().inputs.push(input);
        }

        for o in text.outputs {
            let output = Output::new(Arc::clone(&sequence), o)?;
            sequence.write().unwrap().outputs.push(output);
        }

        for t in text.treatments {
            let treatment = Treatment::new(Arc::clone(&sequence), t)?;
            sequence.write().unwrap().treatments.push(treatment);
        }

        for c in text.connections {
            let connection = Connection::new(Arc::clone(&sequence), c)?;
            sequence.write().unwrap().connections.push(connection);
        }

        Ok(sequence)
    }

    /// Search for a declared model.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("AudioToHpcpImage").unwrap().read().unwrap();
    /// 
    /// let audio_manager = borrowed_sequence.find_declared_model("AudioManager");
    /// let dont_exist = borrowed_sequence.find_declared_model("DontExist");
    /// assert!(audio_manager.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_declared_model(&self, name: & str) -> Option<&Arc<RwLock<DeclaredModel>>> {
        self.declared_models.iter().find(|&m| m.read().unwrap().name == name) 
    }

    /// Search for an instancied model.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("Main").unwrap().read().unwrap();
    /// 
    /// let audio = borrowed_sequence.find_instancied_model("Audio");
    /// let dont_exist = borrowed_sequence.find_instancied_model("DontExist");
    /// assert!(audio.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_instancied_model(&self, name: & str) -> Option<&Arc<RwLock<InstanciedModel>>> {
        self.instancied_models.iter().find(|&m| m.read().unwrap().name == name) 
    }

    /// Search for a requirement.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("AudioToHpcpImage").unwrap().read().unwrap();
    /// 
    /// let signal = borrowed_sequence.find_requirement("@Signal");
    /// let dont_exist = borrowed_sequence.find_requirement("@DontExist");
    /// assert!(signal.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_requirement(&self, name: & str) -> Option<&Arc<RwLock<Requirement>>> {
        self.requirements.iter().find(|&r| r.read().unwrap().name == name) 
    }

    /// Search for an input.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("HPCP").unwrap().read().unwrap();
    /// 
    /// let spectrum = borrowed_sequence.find_input("spectrum");
    /// let dont_exist = borrowed_sequence.find_input("dontExist");
    /// assert!(spectrum.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_input(&self, name: & str) -> Option<&Arc<RwLock<Input>>> {
        self.inputs.iter().find(|&i| i.read().unwrap().name == name) 
    }

    /// Search for an output.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("HPCP").unwrap().read().unwrap();
    /// 
    /// let hpcp = borrowed_sequence.find_output("hpcp");
    /// let dont_exist = borrowed_sequence.find_output("dontExist");
    /// assert!(hpcp.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_output(&self, name: & str) -> Option<&Arc<RwLock<Output>>> {
        self.outputs.iter().find(|&o| o.read().unwrap().name == name) 
    }

    /// Search for a treatment.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("Spectrum").unwrap().read().unwrap();
    /// 
    /// let core_frame_cutter = borrowed_sequence.find_treatment("CoreFrameCutter");
    /// let dont_exist = borrowed_sequence.find_treatment("DontExist");
    /// assert!(core_frame_cutter.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn find_treatment(&self, name: & str) -> Option<&Arc<RwLock<Treatment>>> {
        self.treatments.iter().find(|&t| t.read().unwrap().name == name) 
    }
}

impl Node for Sequence {
    
    fn make_references(&mut self, path: &Path) -> Result<(), ScriptError> {

        self.identifier = path.to_identifier(&self.name);

        Ok(())
    }

    fn children(&self) -> Vec<Arc<RwLock<dyn Node>>> {

        let mut children: Vec<Arc<RwLock<dyn Node>>> = Vec::new();

        self.declared_models.iter().for_each(|m| children.push(Arc::clone(&m) as Arc<RwLock<dyn Node>>));
        self.parameters.iter().for_each(|p| children.push(Arc::clone(&p) as Arc<RwLock<dyn Node>>));
        self.instancied_models.iter().for_each(|m| children.push(Arc::clone(&m) as Arc<RwLock<dyn Node>>));
        self.requirements.iter().for_each(|r| children.push(Arc::clone(&r) as Arc<RwLock<dyn Node>>));
        self.inputs.iter().for_each(|i| children.push(Arc::clone(&i) as Arc<RwLock<dyn Node>>));
        self.outputs.iter().for_each(|o| children.push(Arc::clone(&o) as Arc<RwLock<dyn Node>>));
        self.treatments.iter().for_each(|t| children.push(Arc::clone(&t) as Arc<RwLock<dyn Node>>));
        self.connections.iter().for_each(|c| children.push(Arc::clone(&c) as Arc<RwLock<dyn Node>>));

        children
    }
}

impl DeclarativeElement for Sequence {

    fn declarative_element(&self) -> DeclarativeElementType {
        DeclarativeElementType::Sequence(&self)
    }

    /// Search for a parameter.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// # use melodium_rust::script::semantic::declarative_element::DeclarativeElement;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_sequence = borrowed_script.find_sequence("Spectrum").unwrap().read().unwrap();
    /// 
    /// let frame_size = borrowed_sequence.find_declared_parameter("frameSize");
    /// let dont_exist = borrowed_sequence.find_declared_parameter("dontExist");
    /// assert!(frame_size.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    fn find_declared_parameter(&self, name: & str) -> Option<&Arc<RwLock<DeclaredParameter>>> {
        self.parameters.iter().find(|&p| p.read().unwrap().name == name)
    }
}
