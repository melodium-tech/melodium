use super::{DataTrait, Documented, Identified};
use core::fmt::{Debug, Display};
use std::sync::Arc;

pub trait Object: Identified + Documented + Display + Debug + Send + Sync {
    fn implements(&self) -> &[DataTrait];
    fn as_identified(&self) -> Arc<dyn Identified>;
}

impl PartialEq for dyn Object {
    fn eq(&self, other: &Self) -> bool {
        self.identifier() == other.identifier()
    }
}
