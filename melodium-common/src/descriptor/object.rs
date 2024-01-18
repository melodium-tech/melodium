use super::{DataTrait, Documented, Identified};
use core::fmt::{Debug, Display};

pub trait Object: Identified + Documented + Display + Debug + Send + Sync {
    fn implements(&self) -> &[DataTrait];
}

impl PartialEq for dyn Object {
    fn eq(&self, other: &Self) -> bool {
        self.identifier() == other.identifier()
    }
}
