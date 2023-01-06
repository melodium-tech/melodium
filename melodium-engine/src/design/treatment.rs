
use core::fmt::{Debug};
use super::{Parameter, ModelInstanciation, TreatmentInstanciation, Connection};
use crate::descriptor::Treatment as TreatmentDescriptor;
use std::sync::Weak;
use std::collections::{HashMap};

#[derive(Debug)]
pub struct Treatment {
    pub descriptor: Weak<TreatmentDescriptor>,
    pub model_instanciations: HashMap<String, ModelInstanciation>,
    pub treatments: HashMap<String, TreatmentInstanciation>,
    pub connections: Vec<Connection>,
}
