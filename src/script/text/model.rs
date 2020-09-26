
//! Module dedicated to [Model](struct.Model.html) parsing.

use crate::script::error::ScriptError;

use super::PositionnedString;
use super::word::{expect_word, expect_word_kind, Kind, Word};
use super::common::parse_parameters_declarations;
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
    ///     layers = layers
    ///     function = function
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

        // We parse declarations.
        expect_word_kind(Kind::OpeningParenthesis, "Parameters declaration expected '('.", &mut iter)?;
        let parameters = parse_parameters_declarations(&mut iter)?;

        // The model type.
        expect_word_kind(Kind::Colon, "Model type declaration expected ':'.", &mut iter)?;
        let r#type = expect_word_kind(Kind::Name, "Model type expected.", &mut iter)?;
        
        // And then the internal assignations.
        expect_word_kind(Kind::OpeningBrace, "Model content declaration expected '{'.", &mut iter)?;

        let mut assignations = Vec::new();

        loop {

            let word = expect_word("Unexpected end of script.", &mut iter)?;

            if word.kind == Some(Kind::ClosingBrace) {
                break;
            }
            else if word.kind == Some(Kind::Name) {

                expect_word_kind(Kind::Equal, "Component value expected.", &mut iter)?;
                assignations.push(Parameter::build_from_value(PositionnedString{string: word.text, position: word.position}, &mut iter)?);

            }
            else {
                return Err(ScriptError::word("Model content declaration or end '}' expected.".to_string(), word.text, word.position));
            }
        }

        Ok(Self {
            name,
            parameters,
            r#type,
            assignations,
        })

    }
}
