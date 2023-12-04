//! Module dedicated to [Annotation](Annotation) parsing.

use super::PositionnedString;
use melodium_common::descriptor::Attribute;
use regex::Regex;

/// Structure describing an annotation.
///
/// It actually does nothing more than holding the textual annotation itself.
#[derive(Clone, Debug)]
pub struct Annotation {
    pub text: PositionnedString,
}

impl Annotation {
    pub fn as_attribute(&self) -> Option<(String, Attribute)> {
        lazy_static! {
            static ref REGEX_CONTEXT: Regex = Regex::new(r"^#\[(\w+)\((.*)\)\]").unwrap();
        }
        if let Some(cap) = REGEX_CONTEXT.captures(&self.text.string) {
            Some((
                cap.get(1).unwrap().as_str().to_string(),
                cap.get(2).unwrap().as_str().to_string(),
            ))
        } else {
            None
        }
    }
}

/// Structure handling annotations, documentation, and comments.
#[derive(Clone, Debug)]
pub struct CommentsAnnotations {
    pub doc: Option<PositionnedString>,
    pub comments: Vec<PositionnedString>,
    pub annotations: Vec<Annotation>,
}
