use core::fmt::{Debug, Display};
use downcast_rs::{impl_downcast, DowncastSync};
use super::DataTrait;

#[derive(Clone, Hash, Debug)]
pub struct Generic {
    pub name: String,
    pub traits: Vec<DataTrait>,
}

impl Generic {
    pub fn new(name: String, traits: Vec<DataTrait>) -> Self {
        Self {
            name,
            traits,
        }
    }
}

pub trait Generics: DowncastSync + Display + Debug + Send + Sync {
    fn generics(&self) -> &Vec<Generic>;
}
impl_downcast!(sync Generics);
