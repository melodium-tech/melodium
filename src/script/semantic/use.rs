
//! Module dedicated to Use semantic analysis.

use super::common::Node;

use std::sync::{Arc, Weak, RwLock};
use crate::script::error::ScriptError;
use crate::script::text::Use as TextUse;
use crate::script::path::{Path, PathRoot};
use crate::logic::descriptor::identifier::{Identifier, Root};

use super::script::Script;

/// Structure managing and describing semantic of a use.
/// 
/// It owns the whole [text use](../../text/use/struct.Use.html).
#[derive(Debug)]
pub struct Use {
    pub text: TextUse,

    pub script: Weak<RwLock<Script>>,

    pub path: Path,
    pub element: String,
    pub r#as: String,

    pub identifier: Option<Identifier>,
}

impl Use {
    /// Create a new semantic use, based on textual use.
    /// 
    /// * `script`: the parent script that "owns" this use.
    /// * `text`: the textual use.
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
    /// # use melodium::script::path::Path;
    /// let address = "melodium-tests/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// // Internally, Script::new call Use::new(Rc::clone(&script), text_use)
    /// 
    /// let borrowed_script = script.read().unwrap();
    /// let borrowed_use = borrowed_script.find_use("CoreSpectrum").unwrap().read().unwrap();
    /// 
    /// assert_eq!(borrowed_use.path, Path::new(vec!["core".to_string(), "signal".to_string()]));
    /// assert_eq!(borrowed_use.element, "Spectrum");
    /// assert_eq!(borrowed_use.r#as, "CoreSpectrum");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(script: Arc<RwLock<Script>>, text: TextUse) -> Result<Arc<RwLock<Self>>, ScriptError> {

        let r#as;
        if let Some(ps) = &text.r#as {
            r#as = ps;
        }
        else {
            r#as = &text.element;
        }

        {
            let borrowed_script = script.read().unwrap();

            let r#use = borrowed_script.find_use(&r#as.string);
            if r#use.is_some() {
                return Err(ScriptError::semantic("'".to_string() + &r#as.string + "' is already used.", r#as.position))
            }
        }

        let path = Path::new(text.path.iter().map(|i| i.string.clone()).collect());

        Ok(Arc::<RwLock<Self>>::new(RwLock::new(Self {
            script: Arc::downgrade(&script),
            path,
            element: text.element.string.clone(),
            r#as: r#as.string.clone(),
            text,
            identifier: None,
        })))
    }
}

impl Node for Use {
    fn make_references(&mut self, path: &Path) -> Result<(), ScriptError> {

        if self.path.root() == PathRoot::Other {
            Err(ScriptError::semantic("Root '".to_string() + self.path.path().first().unwrap_or(&"".to_string()) + "' is not valid.", self.text.element.position))
        }
        else {
            if let Some(root) = match self.path.root() {
                PathRoot::Core => Some(Root::Core),
                PathRoot::Std => Some(Root::Std),
                PathRoot::Main => Some(Root::Main),
                _ => None
            } { // "Non-local" case

                self.identifier = Some(Identifier::new(root, self.path.path().clone(), &self.element));

                Ok(())
            }
            else { // "Local" case
                
                let mut steps = path.path().clone();
                steps.append(&mut self.path.path().clone());

                let root = match path.root() {
                    PathRoot::Std => Some(Root::Std),
                    PathRoot::Main => Some(Root::Main),
                    _ => None
                };

                self.identifier = Some(Identifier::new(root.unwrap(), steps, &self.element));

                Ok(())
            }
        }

        
    }
}
