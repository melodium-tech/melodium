use crate::executive::RecvResult;
use async_trait::async_trait;
use core::fmt::Debug;

use super::{TransmissionValue, Value};

#[async_trait]
pub trait Input: Debug + Send + Sync {
    fn close(&self);

    async fn recv_many(&self) -> RecvResult<TransmissionValue>;
    async fn recv_one(&self) -> RecvResult<Value>;
}
