
use crate::script::error::ScriptError;

use super::word::{expect_word_kind, Kind, Word};

pub struct Connection {
    pub name_start_point: String,
    pub name_data_out: Option<String>,
    pub name_end_point: String,
    pub name_data_in: Option<String>
}

impl Connection {
    pub fn build_from_name_end_point(name: String, mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let name_end_point = expect_word_kind(Kind::Name, "Connection endpoint name expected.", &mut iter)?;

        Ok(Self {
            name_start_point: name,
            name_data_out: None,
            name_end_point: name_end_point,
            name_data_in: None,
        })
    }

    pub fn build_from_name_data_out(name: String, mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let name_data_out = expect_word_kind(Kind::Name, "Connection data output name expected.", &mut iter)?;
        expect_word_kind(Kind::RightArrow, "Connection arrow expected.", &mut iter)?;
        let name_end_point = expect_word_kind(Kind::Name, "Connection endpoint name expected.", &mut iter)?;
        expect_word_kind(Kind::Dot, "Connection data input expected.", &mut iter)?;
        let name_data_in = expect_word_kind(Kind::Name, "Connection data input name expected.", &mut iter)?;

        Ok(Self {
            name_start_point: name,
            name_data_out: Some(name_data_out),
            name_end_point: name_end_point,
            name_data_in: Some(name_data_in),
        })
    }
}
