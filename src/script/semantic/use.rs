
//! Module dedicated to Use semantic analysis.

use super::common::Node;

use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Use as TextUse;
use crate::script::path::Path;

use super::script::Script;

/// Structure managing and describing semantic of a use.
/// 
/// It owns the whole [text use](../../text/use/struct.Use.html).
pub struct Use {
    pub text: TextUse,

    pub script: Rc<RefCell<Script>>,

    pub path: Path,
    pub element: String,
    pub r#as: String,
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
    /// # use melodium_rust::script::path::Path;
    /// let address = "examples/semantic/simple_build.mel";
    /// let mut raw_text = String::new();
    /// # let mut file = File::open(address).unwrap();
    /// # file.read_to_string(&mut raw_text);
    /// 
    /// let text_script = TextScript::build(&raw_text)?;
    /// 
    /// let script = Script::new(text_script)?;
    /// // Internally, Script::new call Use::new(Rc::clone(&script), text_use)
    /// 
    /// let borrowed_script = script.borrow();
    /// let borrowed_use = borrowed_script.find_use("Spectrum").unwrap().borrow();
    /// 
    /// assert_eq!(borrowed_use.path, Path::new(vec!["core".to_string(), "signal".to_string()]));
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn new(script: Rc<RefCell<Script>>, text: TextUse) -> Result<Rc<RefCell<Self>>, ScriptError> {

        let r#as;
        if let Some(ps) = &text.r#as {
            r#as = ps;
        }
        else {
            r#as = &text.element;
        }

        {
            let borrowed_script = script.borrow();

            let r#use = borrowed_script.find_use(&r#as.string);
            if r#use.is_some() {
                return Err(ScriptError::semantic("'".to_string() + &r#as.string + "' is already used.", r#as.position))
            }
        }

        let path = Path::new(text.path.iter().map(|i| i.string.clone()).collect());

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self {
            script,
            path,
            element: text.element.string.clone(),
            r#as: r#as.string.clone(),
            text,
        })))
    }
}

impl Node for Use {
    
}
