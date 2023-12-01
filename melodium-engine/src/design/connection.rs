use melodium_common::descriptor::{Attributes, Attribuable};

#[derive(Clone, Debug)]
pub enum IO {
    Sequence(),
    Treatment(String),
}

#[derive(Clone, Debug)]
pub struct Connection {
    pub output_treatment: IO,
    pub output_name: String,

    pub input_treatment: IO,
    pub input_name: String,

    pub attributes: Attributes,
}

impl Attribuable for Connection {
    fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}
