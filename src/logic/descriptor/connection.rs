
use super::datatype::DataType;

#[derive(Debug)]
pub struct Connection {

    output_type: Option<DataType>,
    input_type: Option<DataType>,
}

impl Connection {
    pub fn new(output_type: Option<DataType>, input_type: Option<DataType>) -> Self {
        Self {
            output_type,
            input_type,
        }
    }

    pub fn data_transmission(&self) -> bool {
        self.output_type.is_some() || self.input_type.is_some()
    }

    pub fn output_type(&self) -> &Option<DataType> {
        &self.output_type
    }

    pub fn input_type(&self) -> &Option<DataType> {
        &self.input_type
    }
}
