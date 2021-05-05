
use crate::executive::data::Data;

#[derive(Clone, Debug)]
pub enum Value {
    Raw(Data),
    Variable(String),
    Context((String, String)),
}
