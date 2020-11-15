
#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(String),
}
