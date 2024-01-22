use super::{DataTrait, Documented, Identified};
use core::fmt::{Debug, Display};

pub trait Data: Identified + Documented + Display + Debug + Send + Sync {
    fn implements(&self) -> &[DataTrait];
}

impl PartialEq for dyn Data {
    fn eq(&self, other: &Self) -> bool {
        self.identifier() == other.identifier()
    }
}
