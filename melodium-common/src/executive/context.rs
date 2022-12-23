
use crate::executive::Value;

pub trait Context {
    fn get_value(&self, name: &str) -> Option<&Value>;
}
