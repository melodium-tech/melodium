use super::Value;
use melodium_common::descriptor::Identifier;

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub value: Value,
}

impl Parameter {
    pub fn make_use(&self, identifier: &Identifier) -> bool {
        self.value.make_use(identifier)
    }

    pub fn uses(&self) -> Vec<Identifier> {
        self.value.uses()
    }
}
