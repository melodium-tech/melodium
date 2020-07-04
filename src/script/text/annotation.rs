
//! Module dedicated to [Annotation](struct.Annotation.html) parsing.

use super::PositionnedString;

/// Structure describing an annotation.
/// 
/// It actually does nothing more than holding the textual annotation itself.
#[derive(Clone)]
pub struct Annotation {
    pub text: PositionnedString,
}
