
use super::identified::Identified;
use super::identifier::Identifier;

pub struct ModelConfig {
    identifier: Identifier,
}

impl Identified for ModelConfig {
    fn identifier(&self) -> &Identifier {
        &self.identifier
    }
}
