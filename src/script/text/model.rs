
//! Module dedicated to [Model](struct.Model.html) parsing.

use crate::script::error::ScriptError;

use super::PositionnedString;
use super::word::{expect_word_kind, Kind, Word};

/// Structure describing a textual model.
/// 
/// It owns a name, and a type (model type, not [data type](../type/struct.Type.html)).
#[derive(Clone)]
pub struct Model {
    pub name: PositionnedString,
    pub r#type: PositionnedString,
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
    /// model MachineLearningModel(SparseAutoencoder)
    /// {
    /// }
    /// "##;
    /// 
    /// let words = get_words(text).unwrap();
    /// let mut iter = words.iter();
    /// 
    /// let model_keyword = expect_word_kind(Kind::Name, "Keyword expected.", &mut iter)?;
    /// assert_eq!(model_keyword, "model");
    /// 
    /// let model = Model::build(&mut iter)?;
    /// 
    /// assert_eq!(model.name, "MachineLearningModel");
    /// assert_eq!(model.r#type, "SparseAutoencoder");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build(mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let name = expect_word_kind(Kind::Name, "Model name expected.", &mut iter)?;
        expect_word_kind(Kind::OpeningParenthesis, "Parameters declaration expected '('.", &mut iter)?;
        let r#type = expect_word_kind(Kind::Name, "Model type expected.", &mut iter)?;
        expect_word_kind(Kind::ClosingParenthesis, "End of parameters declaration expected ')'.", &mut iter)?;
        expect_word_kind(Kind::OpeningBrace, "Model content declaration expected '{'.", &mut iter)?;
        expect_word_kind(Kind::ClosingBrace, "End of model content declaration expected '}'.", &mut iter)?;

        Ok(Self {
            name,
            r#type,
        })

    }
}
