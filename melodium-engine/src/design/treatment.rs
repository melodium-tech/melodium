use super::{Connection, ModelInstanciation, TreatmentInstanciation};
use crate::descriptor::Treatment as TreatmentDescriptor;
use core::fmt::Debug;
use std::collections::HashMap;
use std::sync::Weak;

#[derive(Debug, Clone)]
pub struct Treatment {
    pub descriptor: Weak<TreatmentDescriptor>,
    pub model_instanciations: HashMap<String, ModelInstanciation>,
    pub treatments: HashMap<String, TreatmentInstanciation>,
    pub connections: Vec<Connection>,
}
