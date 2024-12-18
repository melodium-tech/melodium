use super::{Attribuable, Identifier};
use core::fmt::Debug;

pub trait Identified: Attribuable + Debug + Send + Sync {
    fn identifier(&self) -> &Identifier;
    /// Tells if the identified element uses element with given identifier.
    ///
    /// ℹ️ The element do not consider that it uses itself.
    fn make_use(&self, identifier: &Identifier) -> bool;
    /// Gives list of identifiers the element uses.
    ///
    /// ℹ️ The element do not uses itself, so its own identifier is not included in the list.
    fn uses(&self) -> Vec<Identifier>;
}
