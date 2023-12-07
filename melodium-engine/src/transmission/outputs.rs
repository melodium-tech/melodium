use crate::transmission::{BlindOutput, Output};
use melodium_common::executive::{Output as ExecutiveOutput, Outputs as OutputsTrait};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Outputs {
    outputs: HashMap<String, Output>,
}

impl Outputs {
    pub fn new(outputs: HashMap<String, Output>) -> Self {
        Self { outputs }
    }
}

impl OutputsTrait for Outputs {
    fn get(&mut self, output: &str) -> Box<dyn ExecutiveOutput> {
        self.outputs
            .remove(output)
            .map(|output| Box::new(output) as Box<dyn ExecutiveOutput>)
            .unwrap_or_else(|| Box::new(BlindOutput::new()))
    }
}
