
use crate::executive::data::Data;

#[derive(Clone)]
pub enum Value {
    Raw(Data),
    Variable(String),
    Context((String, String)),
}
