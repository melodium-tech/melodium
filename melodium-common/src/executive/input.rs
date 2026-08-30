use crate::executive::{RecvResult, TransmissionError};
use async_trait::async_trait;
use core::fmt::Debug;
use std::convert::TryInto;

use super::{GetData, TransmissionValue, Value};

#[async_trait]
pub trait Input: Debug + Send + Sync {
    fn close(&self);

    async fn recv_many(&self) -> RecvResult<TransmissionValue>;
    async fn recv_one(&self) -> RecvResult<Value>;
}

/// Pairs `recv_one`/`recv_many` with a typed cast, so a mismatch between what a graph
/// wired up and what a treatment expects surfaces as `TransmissionError::TypeMismatch`
/// instead of requiring every call site to `.unwrap()` a `GetData`/`TryInto` cast by hand
/// (and panic the track on a mismatch). Blanket-implemented for every `Input`.
#[async_trait]
pub trait InputExt: Input {
    async fn recv_one_as<T: Send>(&self) -> RecvResult<T>
    where
        Value: GetData<T>,
    {
        self.recv_one()
            .await?
            .try_data()
            .map_err(|_| TransmissionError::TypeMismatch)
    }

    async fn recv_many_as<T: Send>(&self) -> RecvResult<Vec<T>>
    where
        TransmissionValue: TryInto<Vec<T>>,
    {
        self.recv_many()
            .await?
            .try_into()
            .map_err(|_| TransmissionError::TypeMismatch)
    }
}

impl<I: Input + ?Sized> InputExt for I {}

#[cfg(test)]
mod input_ext_tests {
    use super::*;
    use std::sync::Mutex;

    // Minimal `Input` yielding one fixed batch, just enough to exercise `InputExt`'s
    // casting logic without pulling in a real channel/engine implementation.
    #[derive(Debug)]
    struct FakeInput {
        batch: Mutex<Option<TransmissionValue>>,
    }

    fn fake_input(batch: TransmissionValue) -> FakeInput {
        FakeInput {
            batch: Mutex::new(Some(batch)),
        }
    }

    #[async_trait]
    impl Input for FakeInput {
        fn close(&self) {
            *self.batch.lock().unwrap() = None;
        }

        async fn recv_many(&self) -> RecvResult<TransmissionValue> {
            self.batch
                .lock()
                .unwrap()
                .take()
                .ok_or(TransmissionError::EverythingClosed)
        }

        async fn recv_one(&self) -> RecvResult<Value> {
            let mut data = self.recv_many().await?;
            data.pop_front().ok_or(TransmissionError::NoData)
        }
    }

    #[test]
    fn recv_one_as_returns_correctly_typed_value() {
        async_std::task::block_on(async {
            let input = fake_input(TransmissionValue::new(Value::String("hello".to_string())));
            let value: String = input.recv_one_as::<String>().await.unwrap();
            assert_eq!(value, "hello");
        });
    }

    #[test]
    fn recv_one_as_reports_type_mismatch_instead_of_panicking() {
        async_std::task::block_on(async {
            let input = fake_input(TransmissionValue::new(Value::String("hello".to_string())));
            let result = input.recv_one_as::<u64>().await;
            assert!(matches!(result, Err(TransmissionError::TypeMismatch)));
        });
    }

    #[test]
    fn recv_many_as_returns_correctly_typed_batch() {
        async_std::task::block_on(async {
            let input = fake_input(TransmissionValue::Byte(std::collections::VecDeque::from(
                vec![1u8, 2u8],
            )));
            let values: Vec<u8> = input.recv_many_as::<u8>().await.unwrap();
            assert_eq!(values, vec![1, 2]);
        });
    }

    #[test]
    fn recv_many_as_reports_type_mismatch_instead_of_panicking() {
        async_std::task::block_on(async {
            let input = fake_input(TransmissionValue::Byte(std::collections::VecDeque::from(
                vec![1u8, 2u8],
            )));
            let result = input.recv_many_as::<String>().await;
            assert!(matches!(result, Err(TransmissionError::TypeMismatch)));
        });
    }
}
