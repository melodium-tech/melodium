
/*
use std::sync::Arc;
use std::collections::HashMap;
use super::data::Data;
use super::model::Model;
use super::super::logic::descriptor::CoreTreatmentDescriptor;

pub struct Treatment {
    descriptor: Arc<CoreTreatmentDescriptor>,
    parameters: HashMap<String, Data>,
    models: HashMap<String, Model>,
    method: fn(HashMap<String, Data>) -> HashMap<String, Data>,
}

pub struct AdditionTreatment {
    vectorA: Vec<i64>,
    vectorB: Vec<i64>,

    method: fn(),

    vectorOut: &fn(Vec<i64>)
}

pub struct Interface<'a> {
    inputs: HashMap<&'a str, &'a fn()>,

    outputs: HashMap<&'a str, &'a fn()>
}
*/
