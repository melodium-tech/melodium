use crate::{
    error::{LogicErrors, LogicResult},
    log::Log,
};
use async_std::channel::Sender;
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
    fn add_logs_listener(&self, sender: Sender<Log>);
    async fn live(&self);
    async fn instanciate(&self, callback: Option<DirectCreationCallback>) -> LogicResult<()>;
    async fn end(&self);
}
