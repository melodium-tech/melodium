
use crate::script::error::ScriptError;

use super::word::{expect_word_kind, Kind, Word};

pub struct Model {
    pub name: String,
    pub r#type: String,
}

impl Model {
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
