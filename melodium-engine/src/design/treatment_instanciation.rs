use super::Parameter;
use core::fmt::Debug;
use melodium_common::descriptor::Treatment;
use std::collections::HashMap;
use std::sync::Weak;

#[derive(Debug)]
pub struct TreatmentInstanciation {
    pub name: String,
    pub descriptor: Weak<dyn Treatment>,
    pub models: HashMap<String, String>,
    pub parameters: HashMap<String, Parameter>,
}
