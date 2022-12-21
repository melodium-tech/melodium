
//! Module dedicated to [Use](struct.Use.html) parsing.

use crate::script::error::ScriptError;

use super::PositionnedString;
use super::word::*;

/// Structure describing a textual use.
/// 
/// It owns the path, as vector of strings (which were separated by slashes `/`), the used element name, and optionally the alias. There is no logical nor existence check at this point.
#[derive(Clone, Debug)]
pub struct Use {
    pub path: Vec<PositionnedString>,
    pub element: PositionnedString,
    pub r#as: Option<PositionnedString>,
}

impl Use {
    /// Build use by parsing words.
    /// 
    /// * `iter`: Iterator over words list, next() being expected to be the beginning of the path.
    /// 
    /// ```
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::word::*;
    /// # use melodium::script::text::r#use::Use;
    /// let words = get_words("use path/where/is::Element as MyElement").unwrap();
    /// let mut iter = words.iter();
    /// 
    /// let use_keyword = expect_word_kind(Kind::Name, "Keyword expected.", &mut iter)?;
    /// assert_eq!(use_keyword.string, "use");
    /// 
    /// let r#use = Use::build(&mut iter)?;
    /// 
    /// assert_eq!(r#use.path.iter().map(|p| p.string.clone()).collect::<Vec<String>>(), vec!["path", "where", "is"]);
    /// assert_eq!(r#use.element.string, "Element");
    /// assert_eq!(r#use.r#as.unwrap().string, "MyElement");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build(mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {
        let mut path = Vec::new();
        let element;
        let use_as;

        loop {
            let name = expect_word_kind(Kind::Name, "Path name expected.", &mut iter)?;
            path.push(name);

            let delimiter = expect_word("Unexpected end of script.", &mut iter)?;
            if delimiter.kind == Some(Kind::Slash) {
                continue;
            }
            else if delimiter.kind == Some(Kind::Colon) {
                expect_word_kind(Kind::Colon, "Double colon expected.", &mut iter)?;

                let designation = expect_word("Element name expected.", &mut iter)?;
                let expected_kind;
                if designation.kind == Some(Kind::Name) {
                    expected_kind = Kind::Name;
                    element = PositionnedString { string: designation.text, position: designation.position };
                }
                else if designation.kind == Some(Kind::Function) {
                    expected_kind = Kind::Function;
                    element = PositionnedString { string: designation.text, position: designation.position };
                }
                else {
                    return Err(ScriptError::word("Element name expected.".to_string(), designation.text, designation.position));
                }

                // We check if we are in "use as" case, _cloning_ the iterator in case next word is not about us.
                let possible_as = expect_word_kind(Kind::Name, "", &mut iter.clone());
                if possible_as.is_ok() && possible_as.unwrap().string == "as" {
                    // We discard "as".
                    iter.next();

                    use_as = Some(expect_word_kind(expected_kind, "Alias name expected.", &mut iter)?);
                }
                else {
                    use_as = None;
                }

                break;
            }
            else {
                return Err(ScriptError::word("Slash or double-colon expected.".to_string(), delimiter.text, delimiter.position));
            }
        }

        Ok(Self{
            path,
            element,
            r#as: use_as,
        })
    }
}
