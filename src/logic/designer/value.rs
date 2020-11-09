
#[derive(Clone)]
pub enum Value {
    Raw(/* To implement with executive value */),
    Variable(String),
    Context((String, String)),
}
