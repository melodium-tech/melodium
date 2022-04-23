
use std::fmt::*;
use super::datatype::DataType;
use super::variability::Variability;
use crate::executive::value::Value;

macro_rules! parameter {
    ($name:expr,$variability:ident,$data_structure:ident,$data_type:ident,$default:expr) => {
        crate::logic::descriptor::parameter::Parameter::new(
            $name,
            $variability,
            datatype!($data_structure,$data_type),
            $default,
        )
    };
}
pub(crate) use parameter;

#[derive(Clone, PartialEq, Debug)]
pub struct Parameter {
    name: String,
    variability: Variability,
    datatype: DataType,
    default: Option<Value>,
}

impl Parameter {
    pub fn new(name: &str, variability: Variability, datatype: DataType, default: Option<Value>) -> Self {
        Self {
            name: name.to_string(),
            variability,
            datatype,
            default,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn variability(&self) -> &Variability {
        &self.variability
    }

    pub fn datatype(&self) -> &DataType {
        &self.datatype
    }

    pub fn default(&self) -> &Option<Value> {
        &self.default
    }
}

impl Display for Parameter {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {

        if let Some(default) = &self.default {
            write!(f, "{} _{}_: `{}` (default `{}`)", self.variability, self.name, self.datatype, default)
        }
        else {
            write!(f, "{} _{}_: `{}`", self.variability, self.name, self.datatype)
        }
        
    }
}
