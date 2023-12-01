use super::{Reference, TreatmentInstanciation};
use melodium_common::descriptor::{Attribuable, Attributes};
use std::sync::{Arc, RwLock, Weak};

#[derive(Debug)]
pub enum IO {
    Sequence(),
    Treatment(Weak<RwLock<TreatmentInstanciation>>),
}

impl PartialEq for IO {
    fn eq(&self, other: &Self) -> bool {
        match self {
            IO::Sequence() => false,
            IO::Treatment(s_t) => match other {
                IO::Sequence() => false,
                IO::Treatment(o_t) => s_t.ptr_eq(o_t),
            },
        }
    }
}

/**
 * Describes designed connection.
 *
 * Main point of attention is about connection logic:
 * - a connection entry point is an _output_;
 * - a connection exit point is an _input_.
 * But:
 * - when a connection starts from `self`, its entry point is the `self` treatment **input**;
 * - when a connection ends to `self`, its exit point is the `self` treatment **output**.
 *
 * In functions, all is always ordered in the connection direction, starting from entry point and finishing to exit point.
 */
#[derive(Debug)]
pub struct Connection {
    pub output_treatment: IO,
    pub output_name: String,

    pub input_treatment: IO,
    pub input_name: String,

    pub attributes: Attributes,

    pub design_reference: Option<Arc<dyn Reference>>,
}

impl Connection {
    pub fn new_internal(
        output_name: &str,
        output_treatment: &Arc<RwLock<TreatmentInstanciation>>,
        input_name: &str,
        input_treatment: &Arc<RwLock<TreatmentInstanciation>>,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            output_name: output_name.to_string(),
            output_treatment: IO::Treatment(Arc::downgrade(output_treatment)),
            input_name: input_name.to_string(),
            input_treatment: IO::Treatment(Arc::downgrade(input_treatment)),
            attributes: Attributes::default(),
            design_reference,
        }
    }

    pub fn new_self(
        self_input_name: &str,
        self_output_name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            output_name: self_input_name.to_string(),
            output_treatment: IO::Sequence(),
            input_name: self_output_name.to_string(),
            input_treatment: IO::Sequence(),
            attributes: Attributes::default(),
            design_reference,
        }
    }

    pub fn new_self_to_internal(
        self_input_name: &str,
        input_name: &str,
        input_treatment: &Arc<RwLock<TreatmentInstanciation>>,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            output_name: self_input_name.to_string(),
            output_treatment: IO::Sequence(),
            input_name: input_name.to_string(),
            input_treatment: IO::Treatment(Arc::downgrade(input_treatment)),
            attributes: Attributes::default(),
            design_reference,
        }
    }

    pub fn new_internal_to_self(
        output_name: &str,
        output_treatment: &Arc<RwLock<TreatmentInstanciation>>,
        self_output_name: &str,
        design_reference: Option<Arc<dyn Reference>>,
    ) -> Self {
        Self {
            output_name: output_name.to_string(),
            output_treatment: IO::Treatment(Arc::downgrade(output_treatment)),
            input_name: self_output_name.to_string(),
            input_treatment: IO::Sequence(),
            attributes: Attributes::default(),
            design_reference,
        }
    }
}

impl Attribuable for Connection {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}
