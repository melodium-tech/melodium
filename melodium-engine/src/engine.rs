use crate::{
    debug::{DebugLevel, Event},
    error::{LogicErrors, LogicResult},
};
use async_std::channel::Sender;
use async_trait::async_trait;
use melodium_common::{
    descriptor::{Collection, Identifier},
    executive::{DirectCreationCallback, Level as LogLevel, Log, Value},
};
use std::{collections::HashMap, sync::Arc};

#[async_trait]
pub trait Engine: Send + Sync {
    fn collection(&self) -> Arc<Collection>;
    fn genesis(&self, entry: &Identifier, params: HashMap<String, Value>) -> LogicResult<()>;
    fn errors(&self) -> LogicErrors;
    fn set_auto_end(&self, auto_end: bool);
    fn auto_end(&self) -> bool;
    fn log_level(&self) -> LogLevel;
    fn add_logs_listener(&self, sender: Sender<Log>);
    fn debug_level(&self) -> DebugLevel;
    fn add_debug_listener(&self, sender: Sender<Event>);
    async fn live(&self);
    async fn instanciate(&self, callback: Option<DirectCreationCallback>) -> LogicResult<()>;
    async fn end(&self);
}
