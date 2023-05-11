use crate::error::{LogicErrors, LogicResult};
use melodium_common::descriptor::{Collection, Identifier};
use std::sync::Arc;
pub trait Engine {
    fn collection(&self) -> Arc<Collection>;
    fn genesis(&self, beginning: &Identifier) -> LogicResult<()>;
    fn errors(&self) -> LogicErrors;
    fn live(&self);
    fn end(&self);
}
