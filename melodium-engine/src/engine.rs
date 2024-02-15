use crate::error::{LogicErrors, LogicResult};
use melodium_common::{
    descriptor::{Collection, Identifier},
    executive::Value,
};
use std::{collections::HashMap, sync::Arc};
pub trait Engine: Send + Sync {
    fn collection(&self) -> Arc<Collection>;
    fn genesis(&self, entry: &Identifier, params: HashMap<String, Value>) -> LogicResult<()>;
    fn errors(&self) -> LogicErrors;
    fn live(&self);
    fn end(&self);
}
