use super::Identifier;

pub trait Identified {
    fn identifier(&self) -> &Identifier;
}
