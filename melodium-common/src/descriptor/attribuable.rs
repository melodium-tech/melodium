use core::fmt::Debug;
use std::collections::HashMap;

pub type Attribute = String;

pub type Attributes = HashMap<String, Attribute>;

pub trait Attribuable: Debug + Send + Sync {
    fn attributes(&self) -> &Attributes;
}
