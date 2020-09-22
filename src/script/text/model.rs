
//! Module dedicated to [Model](struct.Model.html) parsing.

use crate::script::error::ScriptError;

use super::PositionnedString;
use super::word::{expect_word_kind, Kind, Word};
use super::common::parse_parameters;
use super::parameter::Parameter;

/// Structure describing a textual model.
/// 
/// It owns a name, parameters, and a type (model type, not [data type](../type/struct.Type.html)).
#[derive(Clone)]
pub struct Model {
    pub name: PositionnedString,
    pub parameters: Vec<Parameter>,
    pub r#type: PositionnedString,
    pub assignations: Vec<Parameter>,
}

impl Model {
    /// Build a model by parsing words.
    /// 
    /// * `iter`: Iterator over words list, next() being expected to be the name.
    ///
    /// # Warning
    /// Models don't support any kind of content in the current implementation. Their specification is not finished.
    /// 
    /// ```
    /// # use melodium_rust::script::error::ScriptError;
    /// # use melodium_rust::script::text::word::*;
    /// # use melodium_rust::script::text::model::Model;
    /// let text = r##"
    /// model MachineLearningModel(layers: Int, function: String = "sigmoid"): SparseAutoencoder
    /// {
    /// }
    /// "##;
    /// 
    /// let words = get_words(text).unwrap();
    /// let mut iter = words.iter();
    /// 
    /// let model_keyword = expect_word_kind(Kind::Name, "Keyword expected.", &mut iter)?;
    /// assert_eq!(model_keyword.string, "model");
    /// 
    /// let model = Model::build(&mut iter)?;
    /// 
    /// assert_eq!(model.name.string, "MachineLearningModel");
    /// assert_eq!(model.parameters.len(), 2);
    /// assert_eq!(model.r#type.string, "SparseAutoencoder");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build(mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let name = expect_word_kind(Kind::Name, "Model name expected.", &mut iter)?;

        let parameters = parse_parameters(&mut iter)?;

        expect_word_kind(Kind::Colon, "Model type declaration expected ':'.", &mut iter)?;
        let r#type = expect_word_kind(Kind::Name, "Model type expected.", &mut iter)?;
        // TODO model type parameters.
        expect_word_kind(Kind::OpeningBrace, "Model content declaration expected '{'.", &mut iter)?;
        expect_word_kind(Kind::ClosingBrace, "End of model content declaration expected '}'.", &mut iter)?;

        Ok(Self {
            name,
            parameters,
            r#type,
            assignations: Vec::new()
        })

    }
}
