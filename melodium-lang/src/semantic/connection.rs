//! Module dedicated to Connection semantic analysis.

use super::common::Node;
use super::common::Reference;
use super::treatment::Treatment;
use super::treatment_instanciation::TreatmentInstanciation;
use crate::error::{wrap_logic_error, ScriptError};
use crate::path::Path;
use crate::text::Connection as TextConnection;
use melodium_engine::designer::Treatment as TreatmentDesigner;
use std::sync::{Arc, RwLock, Weak};

/// Structure managing and describing semantic of a connection.
///
/// It owns the whole [text connection](TextConnection).
#[derive(Debug)]
pub struct Connection {
    pub text: TextConnection,

    pub treatment: Weak<RwLock<Treatment>>,

    pub start_point_self: bool,
    pub start_point: Reference<TreatmentInstanciation>,
    pub end_point_self: bool,
    pub end_point: Reference<TreatmentInstanciation>,

    pub name_data_out: Option<String>,
    pub name_data_in: Option<String>,
}

impl Connection {
    /// Create a new semantic connection, based on textual connection.
    ///
    /// * `treatment`: the parent treatment that owns this connection.
    /// * `text`: the textual connection.
    ///
    /// # Note
    /// Only parent-child relationships are made at this step. Other references can be made afterwards using the [Node trait](Node).
    pub fn new(
        treatment: Arc<RwLock<Treatment>>,
        text: TextConnection,
    ) -> Result<Arc<RwLock<Self>>, ScriptError> {
        if text.name_data_out.is_some() ^ text.name_data_in.is_some() {
            return Err(ScriptError::semantic(
                "Connection from '".to_string()
                    + &text.name_start_point.string
                    + "' to '"
                    + &text.name_end_point.string
                    + "' either transmit data or doesn't, data name is in excess or missing.",
                text.name_start_point.position,
            ));
        }

        let name_data_out;
        if let Some(ndo) = text.name_data_out.clone() {
            name_data_out = Some(ndo.string);
        } else {
            name_data_out = None;
        }

        let name_data_in;
        if let Some(ndi) = text.name_data_in.clone() {
            name_data_in = Some(ndi.string);
        } else {
            name_data_in = None;
        }

        Ok(Arc::<RwLock<Self>>::new(RwLock::new(Self {
            start_point_self: false,
            start_point: Reference::new(text.name_start_point.string.clone()),
            end_point_self: false,
            end_point: Reference::new(text.name_end_point.string.clone()),
            name_data_out,
            name_data_in,
            text,
            treatment: Arc::downgrade(&treatment),
        })))
    }

    pub fn make_design(&self, designer: &mut TreatmentDesigner) -> Result<(), ScriptError> {
        // something to something
        if !self.start_point_self && !self.end_point_self {
            Ok(wrap_logic_error!(
                designer.add_connection(
                    &self.start_point.name,
                    self.name_data_out.as_ref().unwrap(),
                    &self.end_point.name,
                    self.name_data_in.as_ref().unwrap(),
                ),
                self.text.name_start_point.position
            ))
        }
        // Self to something
        else if self.start_point_self && !self.end_point_self {
            Ok(wrap_logic_error!(
                designer.add_input_connection(
                    self.name_data_out.as_ref().unwrap(),
                    &self.end_point.name,
                    self.name_data_in.as_ref().unwrap(),
                ),
                self.text.name_start_point.position
            ))
        }
        // Something to Self
        else if !self.start_point_self && self.end_point_self {
            Ok(wrap_logic_error!(
                designer.add_output_connection(
                    self.name_data_in.as_ref().unwrap(),
                    &self.start_point.name,
                    self.name_data_out.as_ref().unwrap(),
                ),
                self.text.name_start_point.position
            ))
        }
        // Self to Self
        else {
            Ok(wrap_logic_error!(
                designer.add_self_connection(
                    self.name_data_out.as_ref().unwrap(),
                    self.name_data_in.as_ref().unwrap(),
                ),
                self.text.name_start_point.position
            ))
        }
    }
}

impl Node for Connection {
    fn make_references(&mut self, _path: &Path) -> Result<(), ScriptError> {
        let rc_treatment = self.treatment.upgrade().unwrap();
        let treatment = rc_treatment.read().unwrap();

        let treatment_start = treatment.find_treatment_instanciation(&self.start_point.name);
        if treatment_start.is_some() {
            self.start_point.reference = Some(Arc::downgrade(treatment_start.unwrap()));
        } else if self.start_point.name == "Self" {
            self.start_point_self = true;
        } else {
            return Err(ScriptError::semantic(
                "Treatment '".to_string() + &self.start_point.name + "' is unknown.",
                self.text.name_start_point.position,
            ));
        }

        let treatment_end = treatment.find_treatment_instanciation(&self.end_point.name);
        if treatment_end.is_some() {
            self.end_point.reference = Some(Arc::downgrade(treatment_end.unwrap()));
        } else if self.end_point.name == "Self" {
            self.end_point_self = true;
        } else {
            return Err(ScriptError::semantic(
                "Treatment '".to_string() + &self.end_point.name + "' is unknown.",
                self.text.name_end_point.position,
            ));
        }

        Ok(())
    }
}
