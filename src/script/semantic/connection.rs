
use std::rc::Rc;
use std::cell::RefCell;
use crate::script::error::ScriptError;
use crate::script::text::Connection as TextConnection;

use super::sequence::Sequence;
use super::reference::Reference;
use super::treatment::Treatment;

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
    pub fn new(sequence: Rc<RefCell<Sequence>>, text: TextConnection) -> Result<Rc<RefCell<Self>>, ScriptError> {

        if text.name_data_out.is_some() ^ text.name_data_in.is_some() {
            return Err(ScriptError::semantic("Connection from '".to_string() + &text.name_start_point
            + "' to '" + &text.name_end_point + "' either transmit data or doesn't, data name is in excess or missing."))
        }

        Ok(Rc::<RefCell<Self>>::new(RefCell::new(Self {
            start_point: Reference::new(text.name_start_point.clone()),
            end_point: Reference::new(text.name_end_point.clone()),
            data_transmission: text.name_data_out.is_some(),
            name_data_out: text.name_data_out.clone(),
            name_data_in: text.name_data_in.clone(),
            text,
            sequence,
        })))
    }

    pub fn make_references(&mut self) -> Result<(), ScriptError> {

        let sequence = self.sequence.borrow();

        let treatment_start = sequence.find_treatment(&self.start_point.name);
        if treatment_start.is_some() {
            self.start_point.reference = Some(Rc::clone(treatment_start.unwrap()));
        }
        else {
            return Err(ScriptError::semantic("Treatment '".to_string() + &self.start_point.name + "' is unknown."));
        }

        let treatment_end = sequence.find_treatment(&self.end_point.name);
        if treatment_end.is_some() {
            self.end_point.reference = Some(Rc::clone(treatment_end.unwrap()));
        }
        else {
            return Err(ScriptError::semantic("Treatment '".to_string() + &self.end_point.name + "' is unknown."));
        }

        Ok(())
    }
}
