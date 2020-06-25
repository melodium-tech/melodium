
//! Module dedicated to Use semantic analysis.

use super::common::Node;

use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Use as TextUse;

use super::script::Script;

/// Structure managing and describing semantic of a use.
/// 
/// It owns the whole [text use](../../text/use/struct.Use.html).
pub struct Use {
    pub text: TextUse,

    pub script: Rc<RefCell<Script>>,

    pub path: Vec<String>,
    pub file_path: String,
    pub element: String,
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
    /// let script = Script::new(address, text_script)?;
    /// // Internally, Script::new call Use::new(Rc::clone(&script), text_use)
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_use = borrowed_script.find_use("Spectrum").unwrap().borrow();
    /// 
    /// assert_eq!(borrowed_use.file_path, "core/signal");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(script: Rc<RefCell<Script>>, text: TextUse) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let file_path = text.path.join("/");

        {
            let borrowed_script = script.borrow();

            let r#use = borrowed_script.find_use(&text.element);
            if r#use.is_some() {
                return Err(ScriptError::semantic("'".to_string() + &text.element + "' is already used."))
            }
        }

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self {
            script,
            path: text.path.clone(),
            file_path,
            element: text.element.clone(),
            text,
        })))
    }
}

impl Node for Use {
    
}
