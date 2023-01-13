//! Module dedicated to [Connection](struct.Connection.html) parsing.

use super::word::{expect_word_kind, Kind, Word};
use super::PositionnedString;
use crate::ScriptError;

/// Structure describing a textual connection.
///
/// It owns starting point and ending point names, and the names of data associated with, if any.
#[derive(Clone, Debug)]
pub struct Connection {
    pub name_start_point: PositionnedString,
    pub name_data_out: Option<PositionnedString>,
    pub name_end_point: PositionnedString,
    pub name_data_in: Option<PositionnedString>,
}

impl Connection {
    /// Build a connection by parsing words, starting when then end point name is expected.
    ///
    /// * `name`: The name already parsed for the start point (its accuracy is under responsibility of the caller).
    /// * `iter`: Iterator over words list, next() being expected to be the end point name.
    ///
    /// ```
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::word::*;
    /// # use melodium::script::text::connection::Connection;
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
    /// assert_eq!(connection.name_start_point.string, "Feeder");
    /// assert_eq!(connection.name_end_point.string, "Trainer");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build_from_name_end_point(
        name: PositionnedString,
        mut iter: &mut std::slice::Iter<Word>,
    ) -> Result<Self, ScriptError> {
        let name_end_point =
            expect_word_kind(Kind::Name, "Connection endpoint name expected.", &mut iter)?;

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
    /// # use melodium::script::error::ScriptError;
    /// # use melodium::script::text::word::*;
    /// # use melodium::script::text::connection::Connection;
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
    /// assert_eq!(first_connection.name_start_point.string, "AudioFiles");
    /// assert_eq!(first_connection.name_data_out.unwrap().string, "signal");
    /// assert_eq!(first_connection.name_end_point.string, "MakeSpectrum");
    /// assert_eq!(first_connection.name_data_in.unwrap().string, "signal");
    ///
    /// assert_eq!(second_connection.name_start_point.string, "MakeSpectrum");
    /// assert_eq!(second_connection.name_data_out.unwrap().string, "spectrum");
    /// assert_eq!(second_connection.name_end_point.string, "Self");
    /// assert_eq!(second_connection.name_data_in.unwrap().string, "spectrum");
    /// # Ok::<(), ScriptError>(())
    /// ```
    pub fn build_from_name_data_out(
        name: PositionnedString,
        mut iter: &mut std::slice::Iter<Word>,
    ) -> Result<Self, ScriptError> {
        let name_data_out = expect_word_kind(
            Kind::Name,
            "Connection data output name expected.",
            &mut iter,
        )?;
        expect_word_kind(Kind::RightArrow, "Connection arrow expected.", &mut iter)?;
        let name_end_point =
            expect_word_kind(Kind::Name, "Connection endpoint name expected.", &mut iter)?;
        expect_word_kind(Kind::Dot, "Connection data input expected.", &mut iter)?;
        let name_data_in = expect_word_kind(
            Kind::Name,
            "Connection data input name expected.",
            &mut iter,
        )?;

        Ok(Self {
            name_start_point: name,
            name_data_out: Some(name_data_out),
            name_end_point: name_end_point,
            name_data_in: Some(name_data_in),
        })
    }
}
