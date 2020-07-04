
//! Module dedicated to Connection semantic analysis.

use super::common::Node;

use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Connection as TextConnection;

use super::sequence::Sequence;
use super::common::Reference;
use super::treatment::Treatment;

/// Structure managing and describing semantic of a connection.
/// 
/// It owns the whole [text connection](../../text/connection/struct.Connection.html).
pub struct Connection {
    pub text: TextConnection,

    pub sequence: Rc<RefCell<Sequence>>,

    pub start_point: Reference<Treatment>,
    pub end_point: Reference<Treatment>,

    pub data_transmission: bool,
    pub name_data_out: Option<String>,
    pub name_data_in: Option<String>,
}

impl Connection {
    /// Create a new semantic connection, based on textual connection.
    /// 
    /// * `sequence`: the parent sequence that "owns" this connection.
    /// * `text`: the textual connection.
    /// 
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](../common/trait.Node.html).
    pub fn new(sequence: Rc<RefCell<Sequence>>, text: TextConnection) -> Result<Rc<RefCell<Self>>, ScriptError> {

        if text.name_data_out.is_some() ^ text.name_data_in.is_some() {
            return Err(ScriptError::semantic("Connection from '".to_string() + &text.name_start_point.string
            + "' to '" + &text.name_end_point.string + "' either transmit data or doesn't, data name is in excess or missing.",
            text.name_start_point.position))
        }

        let name_data_out;
        if let Some(ndo) = text.name_data_out.clone() {
            name_data_out = Some(ndo.string);
        }
        else {
            name_data_out = None;
        }

        let name_data_in;
        if let Some(ndi) = text.name_data_in.clone() {
            name_data_in = Some(ndi.string);
        }
        else {
            name_data_in = None;
        }

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self {
            start_point: Reference::new(text.name_start_point.string.clone()),
            end_point: Reference::new(text.name_end_point.string.clone()),
            data_transmission: text.name_data_out.is_some(),
            name_data_out,
            name_data_in,
            text,
            sequence,
        })))
    }

}

impl Node for Connection {
    fn make_references(&mut self) -> Result<(), ScriptError> {

        let sequence = self.sequence.borrow();

        let treatment_start = sequence.find_treatment(&self.start_point.name);
        if treatment_start.is_some() {
            self.start_point.reference = Some(Rc::clone(treatment_start.unwrap()));
        }
        else {
            return Err(ScriptError::semantic("Treatment '".to_string() + &self.start_point.name + "' is unknown.", self.text.name_start_point.position));
        }

        let treatment_end = sequence.find_treatment(&self.end_point.name);
        if treatment_end.is_some() {
            self.end_point.reference = Some(Rc::clone(treatment_end.unwrap()));
        }
        else {
            return Err(ScriptError::semantic("Treatment '".to_string() + &self.end_point.name + "' is unknown.", self.text.name_end_point.position));
        }

        Ok(())
    }
}
