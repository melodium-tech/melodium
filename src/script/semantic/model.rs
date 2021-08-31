
//! Module dedicated to Model semantic analysis.

use super::common::Node;

use std::sync::{Arc, Weak, RwLock};
use crate::script::error::ScriptError;
use crate::script::path::Path;
use crate::script::text::Model as TextModel;
use crate::logic::collection_pool::CollectionPool;
use crate::logic::descriptor::identifier::Identifier;
use crate::logic::descriptor::configured_model::ConfiguredModel;
use crate::logic::descriptor::ModelDescriptor;
use crate::logic::designer::ModelDesigner;

use super::script::Script;
use super::declarative_element::{DeclarativeElement, DeclarativeElementType};
use super::declared_parameter::DeclaredParameter;
use super::assignative_element::{AssignativeElement, AssignativeElementType};
use super::assigned_parameter::AssignedParameter;
use super::common::Reference;
use super::r#use::Use;

/// Structure managing and describing semantic of a model.
/// 
/// It owns the whole [text model](../../text/model/struct.Model.html).
pub struct Model {
    pub text: TextModel,

    pub script: Weak<RwLock<Script>>,

    pub name: String,
    pub parameters: Vec<Arc<RwLock<DeclaredParameter>>>,
    pub r#type: Reference<Use>,
    pub assignations: Vec<Arc<RwLock<AssignedParameter>>>,

    pub identifier: Option<Identifier>,

    auto_reference: Weak<RwLock<Self>>,
}

impl Model {
    /// Create a new semantic model, based on textual model.
    /// 
    /// * `script`: the parent script that "owns" this model.
    /// * `text`: the textual model.
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
    /// // Internally, Script::new call Model::new(Arc::clone(&script), text_model)
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_model = borrowed_script.find_model("Files").unwrap().read().unwrap();
    /// 
    /// assert_eq!(borrowed_model.parameters.len(), 1);
    /// assert_eq!(borrowed_model.assignations.len(), 1);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(script: Arc<RwLock<Script>>, text: TextModel) -> Result<Arc<RwLock<Self>>, ScriptError> {

        let model = Arc::<RwLock<Self>>::new(RwLock::new(Self {
            text: text.clone(),
            script: Arc::downgrade(&script),
            name: text.name.string.clone(),
            parameters: Vec::new(),
            r#type: Reference::new(text.r#type.string.clone()),
            assignations: Vec::new(),
            identifier: None,
            auto_reference: Weak::new(),
        }));

        model.write().unwrap().auto_reference = Arc::downgrade(&model);

        {
            let borrowed_script = script.read().unwrap();

            let model = borrowed_script.find_model(&text.name.string);
            if model.is_some() {
                return Err(ScriptError::semantic("'".to_string() + &text.name.string + "' is already declared.", text.name.position))
            }
        }

        for p in text.parameters {
            let declared_parameter = DeclaredParameter::new(Arc::clone(&model) as Arc<RwLock<dyn DeclarativeElement>>, p)?;
            model.write().unwrap().parameters.push(declared_parameter);
        }

        for a in text.assignations {
            let assigned_parameter = AssignedParameter::new(Arc::clone(&model) as Arc<RwLock<dyn AssignativeElement>>, a)?;
            model.write().unwrap().assignations.push(assigned_parameter);
        }

        Ok(model)
    }

    pub fn make_descriptor(&self, collection: &mut CollectionPool) -> Result<(), ScriptError>  {

        if let Some(core_descriptor) = collection.models.get(&self.r#type.reference.as_ref().unwrap().upgrade().unwrap().read().unwrap().identifier.as_ref().unwrap()) {

            let mut descriptor = ConfiguredModel::new(self.identifier.as_ref().unwrap().clone(), &core_descriptor.core_model());

            for rc_parameter in &self.parameters {

                let borrowed_parameter = rc_parameter.read().unwrap();
                let parameter_descriptor = borrowed_parameter.make_descriptor()?;

                descriptor.add_parameter(parameter_descriptor);
            }

            let arc_descriptor = Arc::new(descriptor);
            arc_descriptor.set_autoref(&arc_descriptor);

            collection.models.insert(&(arc_descriptor as Arc<dyn ModelDescriptor>));

            Ok(())
        }
        else {
            Err(ScriptError::semantic("Unknown model \'".to_string() , self.r#type.reference.as_ref().unwrap().upgrade().unwrap().read().unwrap().text.element.position))
        }
    }

    pub fn make_design(&self, collections: &Arc<CollectionPool>) -> Result<(), ScriptError>  {

        let descriptor = collections.models.get(self.identifier.as_ref().unwrap()).unwrap().clone();

        let rc_designer = ModelDesigner::new(collections, &descriptor.downcast_arc::<ConfiguredModel>().unwrap());
        let mut designer = rc_designer.write().unwrap();

        for rc_assignation in &self.assignations {

            let borrowed_assignation = rc_assignation.read().unwrap();

            let assignation_designer = designer.add_parameter(&borrowed_assignation.name).unwrap();

            borrowed_assignation.make_design(&assignation_designer);
        }

        Ok(())
    }
}

impl DeclarativeElement for Model {
    
    fn declarative_element(&self) -> DeclarativeElementType {
        DeclarativeElementType::Model(&self)
    }

    /// Search for a declared parameter.
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
    /// let borrowed_model = borrowed_script.find_model("Files").unwrap().read().unwrap();
    /// 
    /// let directory = borrowed_model.find_declared_parameter("directory");
    /// let dont_exist = borrowed_model.find_declared_parameter("dontExist");
    /// assert!(directory.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    fn find_declared_parameter(&self, name: & str) -> Option<&Arc<RwLock<DeclaredParameter>>> {
        self.parameters.iter().find(|&p| p.read().unwrap().name == name)
    }

}

impl AssignativeElement for Model {
    
    fn assignative_element(&self) -> AssignativeElementType {
        AssignativeElementType::Model(&self)
    }

    fn associated_declarative_element(&self) -> Arc<RwLock<dyn DeclarativeElement>> {
        self.auto_reference.upgrade().unwrap()
    }

    /// Search for an assigned parameter.
    /// 
    /// # Example
    /// ```
    /// # use std::fs::File;
    /// # use std::io::Read;
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::script::Script as TextScript;
    /// # use melodium_rust::script::semantic::script::Script;
    /// # use melodium_rust::script::semantic::assignative_element::AssignativeElement;
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
    /// let borrowed_model = borrowed_script.find_model("AudioEngine").unwrap().read().unwrap();
    /// 
    /// let sample_rate = borrowed_model.find_assigned_parameter("sampleRate");
    /// let dont_exist = borrowed_model.find_assigned_parameter("dontExist");
    /// assert!(sample_rate.is_some());
    /// assert!(dont_exist.is_none());
    /// # Ok::<(), ScriptError>(())
    /// ```
    fn find_assigned_parameter(&self, name: & str) -> Option<&Arc<RwLock<AssignedParameter>>> {
        self.assignations.iter().find(|&a| a.read().unwrap().name == name)
    }

}

impl Node for Model {
    
    fn make_references(&mut self, path: &Path) -> Result<(), ScriptError> {

        let rc_script = self.script.upgrade().unwrap();
        let borrowed_script = rc_script.read().unwrap();

        let r#use = borrowed_script.find_use(&self.r#type.name);
        if r#use.is_none() {
            return Err(ScriptError::semantic("'".to_string() + &self.r#type.name + "' is unkown.", self.text.r#type.position))
        }

        self.r#type.reference = Some(Arc::downgrade(r#use.unwrap()));

        self.identifier = path.to_identifier(&self.name);

        Ok(())
    }
}
