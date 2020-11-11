
#[derive(Clone, PartialEq)]
pub enum Value {
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(String),
}
