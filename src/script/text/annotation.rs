
//! Module dedicated to [Annotation](struct.Annotation.html) parsing.

/// Structure describing an annotation.
/// 
/// It actually does nothing more than holding the textual annotation itself.
#[derive(Clone)]
pub struct Annotation {
    pub text: String,
}
