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

/// Pairs `send_one`/`send_many` with the ordinary `Into<Value>`/`From<Vec<T>>`
/// conversions, so a treatment sending a plain Rust value never has to name `Value`/
/// `TransmissionValue` itself — symmetric with `InputExt::recv_one_as`/`recv_many_as`.
/// Blanket-implemented for every `Output`.
///
/// `send_many_as` in particular is what makes the auto-packing `From<Vec<T>> for Value`
/// (see `value/data.rs`) actually reach the wire/channel: `values.into()` there routes
/// through `PackedArray::try_from_vec` for a packable `T`, exactly as `send_one_as` does
/// for a single value.
#[async_trait]
pub trait OutputExt: Output {
    async fn send_one_as<T: Into<Value> + Send>(&self, value: T) -> SendResult {
        self.send_one(value.into()).await
    }

    async fn send_many_as<T: Send>(&self, values: Vec<T>) -> SendResult
    where
        TransmissionValue: From<Vec<T>>,
    {
        self.send_many(values.into()).await
    }
}

impl<O: Output + ?Sized> OutputExt for O {}

#[cfg(test)]
mod output_ext_tests {
    use super::*;
    use crate::executive::PackedArray;
    use async_std::sync::Mutex;
    use std::convert::TryInto;

    // Minimal `Output` recording whatever it's given, just enough to exercise
    // `OutputExt`'s conversion logic without a real channel/engine implementation.
    #[derive(Debug)]
    struct FakeOutput {
        sent_one: Mutex<Option<Value>>,
        sent_many: Mutex<Option<TransmissionValue>>,
    }

    fn fake_output() -> FakeOutput {
        FakeOutput {
            sent_one: Mutex::new(None),
            sent_many: Mutex::new(None),
        }
    }

    #[async_trait]
    impl Output for FakeOutput {
        async fn close(&self) {}

        async fn send_many(&self, data: TransmissionValue) -> SendResult {
            *self.sent_many.lock().await = Some(data);
            Ok(())
        }

        async fn send_one(&self, data: Value) -> SendResult {
            *self.sent_one.lock().await = Some(data);
            Ok(())
        }

        async fn force_send(&self) {}
    }

    #[test]
    fn send_one_as_converts_through_into_value() {
        async_std::task::block_on(async {
            let output = fake_output();
            output.send_one_as::<u64>(42).await.unwrap();
            assert_eq!(output.sent_one.lock().await.take(), Some(Value::U64(42)));
        });
    }

    // `send_many_as::<T>(Vec<T>)` sends many *scalar* `T` ticks (the batch counterpart
    // of `recv_many_as::<T>() -> Vec<T>`) - for a packable `T` like `u8`, that's still
    // the ordinary per-tick scalar packing from ticket #120, not `Packed`.
    #[test]
    fn send_many_as_sends_a_batch_of_scalar_ticks() {
        async_std::task::block_on(async {
            let output = fake_output();
            output.send_many_as(vec![1u8, 2, 3]).await.unwrap();
            let sent = output.sent_many.lock().await.take().unwrap();
            assert!(matches!(sent, TransmissionValue::U8(_)));
            let back: Vec<u8> = sent.try_into().unwrap();
            assert_eq!(back, vec![1, 2, 3]);
        });
    }

    // The actual point of auto-packing on the send side: a single packable `Vec<T>`
    // *value* (one array, e.g. one `Stream<Vec<byte>>` tick) sent via `send_one_as`
    // must arrive as `Value::Packed`, with no call site touching `Value`/`PackedArray`.
    #[test]
    fn send_one_as_auto_packs_a_vec_of_a_packable_type() {
        async_std::task::block_on(async {
            let output = fake_output();
            output.send_one_as(vec![1u8, 2, 3]).await.unwrap();
            assert!(matches!(
                output.sent_one.lock().await.take(),
                Some(Value::Packed(PackedArray::U8(_)))
            ));
        });
    }

    // `send_many_as::<Arc<Vec<T>>>` is the batch-of-*arrays* shape - many packed-array
    // ticks, matching a `Stream<Vec<byte>>` (as opposed to `send_many_as::<T>` above,
    // which is many scalar ticks matching a `Stream<byte>`).
    #[test]
    fn send_many_as_packed_arrays_sends_a_batch_of_arrays() {
        async_std::task::block_on(async {
            let output = fake_output();
            output
                .send_many_as(vec![
                    std::sync::Arc::new(vec![1u8, 2, 3]),
                    std::sync::Arc::new(vec![4u8, 5]),
                ])
                .await
                .unwrap();
            let sent = output.sent_many.lock().await.take().unwrap();
            assert!(matches!(sent, TransmissionValue::PackedU8(_)));
            assert_eq!(sent.len(), 2);
        });
    }

    #[test]
    fn send_many_as_still_works_for_a_non_packable_type() {
        async_std::task::block_on(async {
            let output = fake_output();
            output
                .send_many_as(vec!["a".to_string(), "b".to_string()])
                .await
                .unwrap();
            let sent = output.sent_many.lock().await.take().unwrap();
            assert!(matches!(sent, TransmissionValue::String(_)));
        });
    }

    #[test]
    fn send_one_as_packed_array_produces_value_packed() {
        async_std::task::block_on(async {
            let output = fake_output();
            let arr = PackedArray::Byte(std::sync::Arc::new(vec![1u8, 2, 3]));
            output.send_one_as(arr).await.unwrap();
            assert!(matches!(
                output.sent_one.lock().await.take(),
                Some(Value::Packed(PackedArray::Byte(_)))
            ));
        });
    }
}
