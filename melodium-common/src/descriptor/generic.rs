use super::DataTrait;
use core::fmt::{Debug, Display};
use downcast_rs::{impl_downcast, DowncastSync};

#[derive(Clone, Hash, Debug)]
pub struct Generic {
    pub name: String,
    pub traits: Vec<DataTrait>,
}

impl PartialEq for Generic {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
        && self.traits.len() == other.traits.len()
        && !self.traits.iter().any(|tr| !other.traits.contains(tr))
    }
}

impl Display for Generic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.traits.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(
                f,
                "{}: {}",
                self.name,
                self.traits
                    .iter()
                    .map(|tr| tr.to_string())
                    .collect::<Vec<_>>()
                    .join(" + ")
            )
        }
    }
}

impl Generic {
    pub fn new(name: String, traits: Vec<DataTrait>) -> Self {
        Self { name, traits }
    }
}

pub trait Generics: DowncastSync + Display + Debug + Send + Sync {
    fn generics(&self) -> &Vec<Generic>;
}
impl_downcast!(sync Generics);
