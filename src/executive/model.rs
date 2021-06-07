
use std::fmt::Debug;
use std::sync::Arc;
use std::collections::HashMap;
//use super::manager::Manager;
use super::value::Value;
use super::super::logic::descriptor::CoreModelDescriptor;

pub trait Model : Debug {

    // Maybe add reference to descriptor?

    fn set_parameter(&mut self, param: &str, value: &Value);

    fn initialize(&self);
    fn shudown(&self);
}

