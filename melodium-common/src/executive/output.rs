use super::{TransmissionValue, Value};
use crate::executive::SendResult;
use async_trait::async_trait;
use core::fmt::Debug;

pub trait Outputs: Debug + Send + Sync {
    fn get(&mut self, output: &str) -> Box<dyn Output>;
}

#[async_trait]
pub trait Output: Debug + Send + Sync {
    async fn close(&self);

    async fn send_many(&self, data: TransmissionValue) -> SendResult;
    async fn send_one(&self, data: Value) -> SendResult;

    async fn force_send(&self);
}
