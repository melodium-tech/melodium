use crate::error::{LogicErrors, LogicResult};
use async_trait::async_trait;
use melodium_common::{
    descriptor::{Collection, Identifier},
    executive::{DirectCreationCallback, Value},
};
use std::{collections::HashMap, sync::Arc};

#[async_trait]
pub trait Engine: Send + Sync {
    fn collection(&self) -> Arc<Collection>;
    fn genesis(&self, entry: &Identifier, params: HashMap<String, Value>) -> LogicResult<()>;
    fn errors(&self) -> LogicErrors;
    fn set_auto_end(&self, auto_end: bool);
    fn auto_end(&self) -> bool;
    async fn live(&self);
    async fn instanciate(
        &self,
        params: HashMap<String, Value>,
        callback: Option<DirectCreationCallback>,
    ) -> LogicResult<()>;
    async fn end(&self);
}
