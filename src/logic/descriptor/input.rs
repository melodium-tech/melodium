
use super::datatype::DataType;
use super::flow::Flow;

macro_rules! input {
    ($name:expr,$data_structure:ident,$data_type:ident,$flow:ident) => {
        crate::logic::descriptor::input::Input::new(
            $name,
            datatype!($data_structure,$data_type),
            crate::logic::descriptor::flow::Flow::$flow,
        )
    };
}
pub(crate) use input;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Input {
    name: String,
    datatype: DataType,
    flow: Flow,
}

impl Input {
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
