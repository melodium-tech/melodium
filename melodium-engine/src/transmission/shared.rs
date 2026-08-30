use melodium_common::executive::TransmissionValue;
use std::sync::Arc;

/// Reclaims an owned `TransmissionValue` from a batch shared across a fan-out send: free
/// when this is the last reference, a clone otherwise. This is what makes sharing a batch
/// across receivers (instead of deep-cloning it once per receiver) pay off — the clone
/// cost, when unavoidable, is deferred to whichever receiver actually needs its own copy.
pub(crate) fn own(data: Arc<TransmissionValue>) -> TransmissionValue {
    Arc::try_unwrap(data).unwrap_or_else(|shared| (*shared).clone())
}
