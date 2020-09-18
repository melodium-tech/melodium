
use super::identified::Identified;
use super::identifier::Identifier;

pub struct ModelType {
    identifier: Identifier,
}

impl Identified for ModelType {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}
