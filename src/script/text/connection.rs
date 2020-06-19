
//! Module dedicated to [Connection](struct.Connection.html) parsing.

use crate::script::error::ScriptError;

use super::word::{expect_word_kind, Kind, Word};

/// Structure describing a textual connection.
/// 
/// It owns starting point and ending point names, and the names of data associated with, if any.
#[derive(Clone)]
pub struct Connection {
    pub name_start_point: String,
    pub name_data_out: Option<String>,
    pub name_end_point: String,
    pub name_data_in: Option<String>
}

impl Connection {
    /// Build a connection by parsing words, starting when then end point name is expected.
    /// 
    /// * `name`: The name already parsed for the start point (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be the end point name.
    /// 
    /// ```
    /// # use lang_trial::script::error::ScriptError;
    /// # use lang_trial::script::text::word::*;
    /// # use lang_trial::script::text::connection::Connection;
    /// 
    /// let text = r##"Feeder --> Trainer"##;
    /// 
    /// let words = get_words(text).unwrap();
    /// let mut iter = words.iter();
    /// 
    /// let start_point_name = expect_word_kind(Kind::Name, "Name expected.", &mut iter)?;
    /// expect_word_kind(Kind::RightArrow, "Right arrow '->' expected.", &mut iter)?;
    /// 
    /// let connection = Connection::build_from_name_end_point(start_point_name, &mut iter)?;
    /// 
    /// assert_eq!(connection.name_start_point, "Feeder");
    /// assert_eq!(connection.name_end_point, "Trainer");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build_from_name_end_point(name: String, mut iter: &mut std::slice::Iter<Word>) -> Result<Self, ScriptError> {

        let name_end_point = expect_word_kind(Kind::Name, "Connection endpoint name expected.", &mut iter)?;

        Ok(Self {
            name_start_point: name,
            name_data_out: None,
            name_end_point: name_end_point,
            name_data_in: None,
        })
    }

    /// Build a connection by parsing words, starting when then name of data out is expected.
    /// 
    /// * `name`: The name already parsed for the data out (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be the data out name.
    /// 
    /// ```
    /// # use lang_trial::script::error::ScriptError;
    /// # use lang_trial::script::text::word::*;
    /// # use lang_trial::script::text::connection::Connection;
    /// 
    /// let text = r##"AudioFiles.signal -> MakeSpectrum.signal,spectrum -> Self.spectrum"##;
    /// 
    /// let words = get_words(text).unwrap();
    /// let mut iter = words.iter();
    /// 
    /// let first_start_point_name = expect_word_kind(Kind::Name, "Name expected.", &mut iter)?;
    /// expect_word_kind(Kind::Dot, "Dot '.' expected.", &mut iter)?;
    /// 
    /// let first_connection = Connection::build_from_name_data_out(first_start_point_name, &mut iter)?;
    /// 
    /// expect_word_kind(Kind::Comma, "Comma ',' expected.", &mut iter)?;
    /// 
    /// let second_connection = Connection::build_from_name_data_out(first_connection.name_end_point.clone(), &mut iter)?;
    /// 
    /// assert_eq!(first_connection.name_start_point, "AudioFiles");
    /// assert_eq!(first_connection.name_data_out, Some("signal".to_string()));
    /// assert_eq!(first_connection.name_end_point, "MakeSpectrum");
    /// assert_eq!(first_connection.name_data_in, Some("signal".to_string()));
    /// 
    /// assert_eq!(second_connection.name_start_point, "MakeSpectrum");
    /// assert_eq!(second_connection.name_data_out, Some("spectrum".to_string()));
    /// assert_eq!(second_connection.name_end_point, "Self");
    /// assert_eq!(second_connection.name_data_in, Some("spectrum".to_string()));
    /// # Ok::<(), ScriptError>(())
    /// ```
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
