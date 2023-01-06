
use core::fmt::{Debug};
use super::Parameter;
use melodium_common::descriptor::Treatment;
use std::sync::Weak;
use std::collections::{HashMap};

#[derive(Debug)]
pub struct TreatmentInstanciation {
    pub name: String,
    pub descriptor: Weak<dyn Treatment>,
    pub models: HashMap<String, String>,
    pub parameters: HashMap<String, Parameter>,
}
