
use super::identifier::Identifier;

pub trait Identified {
    fn identifier(&self) -> &Identifier;
}
