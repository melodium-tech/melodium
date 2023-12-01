use super::{Identifier, Attribuable};
use core::fmt::Debug;

pub trait Identified: Attribuable + Debug + Send + Sync {
    fn identifier(&self) -> &Identifier;
}
