
use crate::script::error::ScriptError;
use crate::script::text::Connection as TextConnection;

use super::sequence::Sequence;

pub struct Connection<'a> {
    pub text: TextConnection,

    pub sequence: &'a Sequence<'a>,

    pub name_start_point: String,
    pub name_end_point: String,

    pub data_transmission: bool,
    pub name_data_out: Option<String>,
    pub name_data_in: Option<String>,
}

impl<'a> Connection<'a> {
    pub fn new(sequence: &'a Sequence, text: TextConnection) -> Result<Self, ScriptError> {

        if text.name_data_out.is_some() ^ text.name_data_in.is_some() {
            return Err(ScriptError::semantic("Connection from '".to_string() + &text.name_start_point
            + "' to '" + &text.name_end_point + "' either transmit data or doesn't, data name is in excess or missing."))
        }

        Ok(Self {
            text,
            sequence,
            name_start_point: text.name_start_point,
            name_end_point: text.name_end_point,
            data_transmission: text.name_data_out.is_some(),
            name_data_out: text.name_data_out,
            name_data_in: text.name_data_in,
        })
    }
}
