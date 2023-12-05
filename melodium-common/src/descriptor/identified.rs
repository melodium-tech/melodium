use super::{Attribuable, Identifier};
use core::fmt::Debug;

pub trait Identified: Attribuable + Debug + Send + Sync {
    fn identifier(&self) -> &Identifier;
}
