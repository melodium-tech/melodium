use crate::error::LogicError;
use melodium_common::descriptor::{Collection, Identifier};
use std::sync::Arc;
pub trait Engine {
    fn collection(&self) -> Arc<Collection>;
    fn genesis(&self, beginning: &Identifier) -> Result<(), Vec<LogicError>>;
    fn errors(&self) -> Vec<LogicError>;
    fn live(&self);
    fn end(&self);
}
