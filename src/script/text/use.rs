
//! Module dedicated to [Use](struct.Use.html) parsing.

use crate::script::error::ScriptError;

use super::word::*;

/// Structure describing a textual use.
/// 
/// It owns the path, as vector of strings (which were separated by slashes `/`), and the used element name. There is no logical nor existence check at this point.
pub struct Use {
    pub path: Vec<String>,
    pub element: String,
}

impl Use {
    /// Build use by parsing words.
    /// 
    /// * `iter`: Iterator over words list, next() being expected to be the beginning of the path.
    /// 
    /// ```
    /// # use lang_trial::script::error::ScriptError;
    /// # use lang_trial::script::text::word::*;
    /// # use lang_trial::script::text::r#use::Use;
    /// let words = get_words("use path/where/is::Element").unwrap();
    /// let mut iter = words.iter();
    /// 
    /// let use_keyword = expect_word_kind(Kind::Name, "Keyword expected.", &mut iter)?;
    /// assert_eq!(use_keyword, "use");
    /// 
    /// let r#use = Use::build(&mut iter)?;
    /// 
    /// assert_eq!(r#use.path, vec!["path", "where", "is"]);
    /// assert_eq!(r#use.element, "Element");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build(mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {
        let mut path = Vec::new();
        let element;

        loop {
            let name = expect_word_kind(Kind::Name, "Path name expected.", &mut iter)?;
            path.push(name);

            let delimiter = expect_word("Unexpected end of script.", &mut iter)?;
            if delimiter.kind == Some(Kind::Slash) {
                continue;
            }
            else if delimiter.kind == Some(Kind::Colon) {
                expect_word_kind(Kind::Colon, "Double colon expected.", &mut iter)?;
                element = expect_word_kind(Kind::Name, "Element name expected.", &mut iter)?;
                break;
            }
            else {
                return Err(ScriptError::new("Slash or double-colon expected.".to_string(), delimiter.text, delimiter.line, delimiter.line_position, delimiter.absolute_position));
            }
        }

        Ok(Self{
            path,
            element,
        })
    }
}
