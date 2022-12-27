
use std::sync::Arc;
use super::Identified;

pub trait Model {
    fn as_identified(&self) -> Arc<dyn Identified>;
}
