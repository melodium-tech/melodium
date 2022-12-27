
use std::sync::Arc;
use super::Identified;

pub trait Function {
    fn as_identified(&self) -> Arc<dyn Identified>;
}
