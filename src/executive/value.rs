
#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(String),
    VecBoolean(Vec<bool>),
    VecInteger(Vec<i64>),
    VecReal(Vec<f64>),
    VecString(Vec<String>),
}
