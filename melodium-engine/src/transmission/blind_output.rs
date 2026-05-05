use async_trait::async_trait;
use melodium_common::executive::{Output as ExecutiveOutput, TransmissionValue};
use melodium_common::executive::{SendResult, Value};

#[derive(Debug, Clone)]
pub struct BlindOutput {}

impl BlindOutput {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl ExecutiveOutput for BlindOutput {
    async fn close(&self) {}
    async fn send_many(&self, _data: TransmissionValue) -> SendResult {
        Ok(())
    }
    async fn send_one(&self, _data: Value) -> SendResult {
        Ok(())
    }
    async fn force_send(&self) {}
}
