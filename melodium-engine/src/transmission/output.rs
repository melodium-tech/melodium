
use melodium_common::executive::Output as ExecutiveOutput;
use crate::transmission::Input;

#[derive(Debug, Clone)]
pub struct Output {
}

impl Output {
    pub fn add_transmission(&self, input: &Vec<Input>) {
        todo!()
    }
}

impl ExecutiveOutput for Output {}

