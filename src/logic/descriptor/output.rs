
use std::fmt::*;
use super::datatype::DataType;
use super::flow::Flow;

macro_rules! output {
    ($name:expr,$data_structure:ident,$data_type:ident,$flow:ident) => {
        crate::logic::descriptor::output::Output::new(
            $name,
            datatype!($data_structure,$data_type),
            crate::logic::descriptor::flow::Flow::$flow,
        )
    };
}
pub(crate) use output;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Output {
    name: String,
    datatype: DataType,
    flow: Flow,
}

impl Output {
    pub fn new(name: &str, datatype: DataType, flow: Flow) -> Self {
        Self {
            name: name.to_string(),
            datatype,
            flow,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn datatype(&self) -> &DataType {
        &self.datatype
    }

    pub fn flow(&self) -> &Flow {
        &self.flow
    }
}

impl Display for Output {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {

        match self.flow {
            Flow::Block => {
                write!(f, "_{}_: `Block<{}>`", self.name, self.datatype)
            },
            Flow::Stream => {
                write!(f, "_{}_: `Stream<{}>`", self.name, self.datatype)
            }
        }
        
    }
}
