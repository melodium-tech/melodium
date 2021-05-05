
use std::sync::Arc;
use std::collections::HashMap;
use super::manager::Manager;
use super::data::Data;
use super::super::logic::descriptor::CoreModelDescriptor;

pub struct Model {
    descriptor: Arc<CoreModelDescriptor>,
    parameters: HashMap<String, Data>,
    manager: Arc<Manager>
}
