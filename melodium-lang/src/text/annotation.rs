
//! Module dedicated to [Annotation](Annotation) parsing.

use super::PositionnedString;

/// Structure describing an annotation.
/// 
/// It actually does nothing more than holding the textual annotation itself.
#[derive(Clone, Debug)]
pub struct Annotation {
    pub text: PositionnedString,
}
