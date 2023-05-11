//! Module dedicated to Connection semantic analysis.

use super::common::Node;
use super::common::Reference;
use super::treatment::Treatment;
use super::treatment_instanciation::TreatmentInstanciation;
use crate::error::ScriptError;
use crate::path::Path;
use crate::text::Connection as TextConnection;
use crate::ScriptResult;
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

    pub name_data_out: String,
    pub name_data_in: String,
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
    ) -> ScriptResult<Arc<RwLock<Self>>> {
        if text.name_data_out.is_none() || text.name_data_in.is_none() {
            return ScriptResult::new_failure(ScriptError::connection_must_transmit_data(
                143,
                text.name_start_point.clone(),
                text.name_end_point.clone(),
            ));
        }

        let name_data_out = text.name_data_out.as_ref().unwrap().string.clone();
        let name_data_in = text.name_data_in.as_ref().unwrap().string.clone();

        ScriptResult::new_success(Arc::<RwLock<Self>>::new(RwLock::new(Self {
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

    pub fn make_design(&self, designer: &mut TreatmentDesigner) -> ScriptResult<()> {
        // something to something
        if !self.start_point_self && !self.end_point_self {
            ScriptResult::from(designer.add_connection(
                &self.start_point.name,
                &self.name_data_out,
                &self.end_point.name,
                &self.name_data_in,
                Some(self.text.name_start_point.into_ref()),
            ))
        }
        // Self to something
        else if self.start_point_self && !self.end_point_self {
            ScriptResult::from(designer.add_input_connection(
                &self.name_data_out,
                &self.end_point.name,
                &self.name_data_in,
                Some(self.text.name_start_point.into_ref()),
            ))
        }
        // Something to Self
        else if !self.start_point_self && self.end_point_self {
            ScriptResult::from(designer.add_output_connection(
                &self.name_data_in,
                &self.start_point.name,
                &self.name_data_out,
                Some(self.text.name_start_point.into_ref()),
            ))
        }
        // Self to Self
        else {
            ScriptResult::from(designer.add_self_connection(
                &self.name_data_out,
                &self.name_data_in,
                Some(self.text.name_start_point.into_ref()),
            ))
        }
    }
}

impl Node for Connection {
    fn make_references(&mut self, _path: &Path) -> ScriptResult<()> {
        let mut result = ScriptResult::new_success(());
        let rc_treatment = self.treatment.upgrade().unwrap();
        let treatment = rc_treatment.read().unwrap();

        let treatment_start = treatment.find_treatment_instanciation(&self.start_point.name);
        if treatment_start.is_some() {
            self.start_point.reference = Some(Arc::downgrade(treatment_start.unwrap()));
        } else if self.start_point.name == "Self" {
            self.start_point_self = true;
        } else {
            result = result.and_degrade_failure(ScriptResult::new_failure(
                ScriptError::treatment_not_found(144, self.text.name_start_point.clone()),
            ));
        }

        let treatment_end = treatment.find_treatment_instanciation(&self.end_point.name);
        if treatment_end.is_some() {
            self.end_point.reference = Some(Arc::downgrade(treatment_end.unwrap()));
        } else if self.end_point.name == "Self" {
            self.end_point_self = true;
        } else {
            result = result.and_degrade_failure(ScriptResult::new_failure(
                ScriptError::treatment_not_found(145, self.text.name_end_point.clone()),
            ));
        }

        result
    }
}
