
use super::datatype::DataType;

#[derive(Debug)]
pub struct Connection {

    output_type: DataType,
    input_type: DataType,
}

impl Connection {
    pub fn new(output_type: DataType, input_type: DataType) -> Self {
        Self {
            output_type,
            input_type,
        }
    }

    pub fn output_type(&self) -> &DataType {
        &self.output_type
    }

    pub fn input_type(&self) -> &DataType {
        &self.input_type
    }
}
