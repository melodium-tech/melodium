
//! Module dedicated to [ModelInstanciation](struct.ModelInstanciation.html) parsing.

use crate::script::error::ScriptError;

use super::PositionnedString;
use super::word::{expect_word_kind, Kind, Word};
use super::common::parse_parameters_assignations;
use super::parameter::Parameter;

/// Structure describing a model instanciation.
/// 
/// It owns an identifier, and list of [parameters](../parameter/struct.Parameter.html).
#[derive(Clone)]
pub struct ModelInstanciation {
    pub name: PositionnedString,
    pub identifier: PositionnedString,
    pub parameters: Vec<Parameter>,
}

impl ModelInstanciation {
    /// Build a model instanciation by parsing words.
    /// 
    /// * `iter`: Iterator over words list, next() being expected to be the name.
    /// 
    /// ```
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::word::*;
    /// # use melodium_rust::script::text::model_instanciation::ModelInstanciation;
    /// let text = r##"Audio: AudioAccess(sampleRate = 44100, channels = "mono")"##;
    /// 
    /// let words = get_words(text).unwrap();
    /// let mut iter = words.iter();
    /// 
    /// let model_instanciation = ModelInstanciation::build(&mut iter)?;
    /// 
    /// assert_eq!(model_instanciation.name.string, "Audio");
    /// assert_eq!(model_instanciation.identifier.string, "AudioAccess");
    /// assert_eq!(model_instanciation.parameters.len(), 2);
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build(mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let name = expect_word_kind(Kind::Name, "Model declaration expected.", &mut iter)?;
        expect_word_kind(Kind::Colon, "Model instanciation ':' expected.", &mut iter)?;
        let identifier = expect_word_kind(Kind::Name, "Model identifier expected.", &mut iter)?;

        let parameters = parse_parameters_assignations(&mut iter)?;

        Ok(Self {
            name,
            identifier,
            parameters,
        })
    }
}
