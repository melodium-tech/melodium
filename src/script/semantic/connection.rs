
//! Module dedicated to Connection semantic analysis.

use super::common::Node;

use std::sync::{Arc, Weak, RwLock};
use crate::script::error::{ScriptError, wrap_logic_error};
use crate::script::path::Path;
use crate::script::text::Connection as TextConnection;
use crate::logic::designer::{SequenceDesigner, ConnectionDesigner};

use super::sequence::Sequence;
use super::common::Reference;
use super::treatment::Treatment;

/// Structure managing and describing semantic of a connection.
/// 
/// It owns the whole [text connection](../../text/connection/struct.Connection.html).
#[derive(Debug)]
pub struct Connection {
    pub text: TextConnection,

    pub sequence: Weak<RwLock<Sequence>>,

    pub start_point_self: bool,
    pub start_point: Reference<Treatment>,
    pub end_point_self: bool,
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
    pub fn new(sequence: Arc<RwLock<Sequence>>, text: TextConnection) -> Result<Arc<RwLock<Self>>, ScriptError> {

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

        Ok(Arc::<RwLock<Self>>::new(RwLock::new(Self {
            start_point_self: false,
            start_point: Reference::new(text.name_start_point.string.clone()),
            end_point_self: false,
            end_point: Reference::new(text.name_end_point.string.clone()),
            data_transmission: text.name_data_out.is_some(),
            name_data_out,
            name_data_in,
            text,
            sequence: Arc::downgrade(&sequence),
        })))
    }

    pub fn get_designer(&self, designer: &mut SequenceDesigner) -> Result<Arc<RwLock<ConnectionDesigner>>, ScriptError> {

        // Data connection
        if self.data_transmission {

            // something to something
            if !self.start_point_self && !self.end_point_self {

                Ok(wrap_logic_error!(
                    designer.add_connection(
                        &self.start_point.name,
                        self.name_data_out.as_ref().unwrap(),
                        &self.end_point.name,
                        self.name_data_in.as_ref().unwrap(),
                        ),
                    self.text.name_start_point.position)
                )
            }

            // Self to something
            else if self.start_point_self {

                Ok(wrap_logic_error!(
                    designer.add_input_connection(
                        self.name_data_out.as_ref().unwrap(),
                        &self.end_point.name,
                        self.name_data_in.as_ref().unwrap(),
                    ),
                    self.text.name_start_point.position)
                )
            }

            // Something to Self
            else if self.end_point_self {

                Ok(wrap_logic_error!(
                    designer.add_output_connection(
                        self.name_data_in.as_ref().unwrap(),
                        &self.start_point.name,
                        self.name_data_out.as_ref().unwrap(),
                    ),
                    self.text.name_start_point.position)
                )
            }

            else {
                panic!("Impossible data connection case")
            }
        }

        // Void connection
        else {
            Ok(designer.add_void_connection(
                &self.start_point.name,
                &self.end_point.name,
            ).unwrap())
        }

    }

    pub fn make_design(&self, designer: &Arc<RwLock<ConnectionDesigner>>, sequence_designer: &Arc<RwLock<SequenceDesigner>>) -> Result<(), ScriptError> {

        let mut designer = designer.write().unwrap();

        // Data connection
        if self.data_transmission {

            // something to something
            if !self.start_point_self && !self.end_point_self {

                wrap_logic_error!(
                    designer.set_output(sequence_designer.read().unwrap().treatments().get(&self.start_point.name).unwrap(), &self.name_data_out),
                    self.text.name_start_point.position
                );

                wrap_logic_error!(
                    designer.set_input(sequence_designer.read().unwrap().treatments().get(&self.end_point.name).unwrap(), &self.name_data_in),
                    self.text.name_end_point.position
                );
            }

            // Self to something
            else if self.start_point_self {

                wrap_logic_error!(
                    designer.set_self_output(&self.name_data_out),
                    self.text.name_data_out.as_ref().unwrap().position
                );

                wrap_logic_error!(
                    designer.set_input(sequence_designer.read().unwrap().treatments().get(&self.end_point.name).unwrap(), &self.name_data_in),
                    self.text.name_end_point.position
                );
            }

            // Something to Self
            else if self.end_point_self {

                wrap_logic_error!(
                    designer.set_output(sequence_designer.read().unwrap().treatments().get(&self.start_point.name).unwrap(), &self.name_data_out),
                    self.text.name_start_point.position
                );

                wrap_logic_error!(
                    designer.set_self_input(&self.name_data_in),
                    self.text.name_data_in.as_ref().unwrap().position
                );
            }
        }

        // Void connection
        else {

            wrap_logic_error!(
                designer.set_output(sequence_designer.read().unwrap().treatments().get(&self.start_point.name).unwrap(), &None),
                self.text.name_start_point.position
            );

            wrap_logic_error!(
                designer.set_input(sequence_designer.read().unwrap().treatments().get(&self.end_point.name).unwrap(), &None),
                self.text.name_end_point.position
            );
        }

        wrap_logic_error!(designer.validate(), self.text.name_start_point.position);

        Ok(())

    }

}

impl Node for Connection {
    fn make_references(&mut self, path: &Path) -> Result<(), ScriptError> {

        let rc_sequence = self.sequence.upgrade().unwrap();
        let sequence = rc_sequence.read().unwrap();

        let treatment_start = sequence.find_treatment(&self.start_point.name);
        if treatment_start.is_some() {
            self.start_point.reference = Some(Arc::downgrade(treatment_start.unwrap()));
        }
        else if self.start_point.name == "Self" {
            self.start_point_self = true;
        }
        else {
            return Err(ScriptError::semantic("Treatment '".to_string() + &self.start_point.name + "' is unknown.", self.text.name_start_point.position));
        }

        let treatment_end = sequence.find_treatment(&self.end_point.name);
        if treatment_end.is_some() {
            self.end_point.reference = Some(Arc::downgrade(treatment_end.unwrap()));
        }
        else if self.end_point.name == "Self" {
            self.end_point_self = true;
        }
        else {
            return Err(ScriptError::semantic("Treatment '".to_string() + &self.end_point.name + "' is unknown.", self.text.name_end_point.position));
        }

        // In case 'Self' is used but no data are transmitted.
        if (self.start_point_self || self.end_point_self) && !self.data_transmission {
            return Err(ScriptError::semantic("Connection with 'Self' require data to be transmitted.".to_string(), self.text.name_start_point.position));
        }

        Ok(())
    }
}
