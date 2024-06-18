use crate::error::{LogicErrors, LogicResult};
use async_trait::async_trait;
use melodium_common::{
    descriptor::{Collection, Identifier},
    executive::{DirectCreationCallback, Input, Value},
};
use std::{collections::HashMap, sync::Arc};

#[async_trait]
pub trait Engine: Send + Sync {
    fn collection(&self) -> Arc<Collection>;
    fn genesis(&self, entry: &Identifier, params: HashMap<String, Value>) -> LogicResult<()>;
    fn errors(&self) -> LogicErrors;
    fn new_input(&self) -> Box<dyn Input>;
    async fn live(&self);
    async fn instanciate(&self, callback: Option<DirectCreationCallback>);
    fn end(&self);
}
