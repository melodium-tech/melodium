use super::Identifier;
use core::fmt::Debug;

pub trait Identified: Debug + Send + Sync {
    fn identifier(&self) -> &Identifier;
}
