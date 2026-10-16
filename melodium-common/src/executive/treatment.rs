use crate::descriptor::{DataType, Treatment as TreatmentDescriptor};
use crate::executive::{Input, Model, Output, SecretsAccess, TrackFuture, Value, World};
use core::fmt::Debug;
use core::future::Future;
use core::pin::Pin;
use std::sync::{Arc, Weak};

pub trait Treatment: Debug + Sync + Send {
    fn descriptor(&self) -> Arc<dyn TreatmentDescriptor>;

    fn set_generic(&self, generic: &str, data_type: DataType);
    fn set_parameter(&self, param: &str, value: Value);
    fn set_model(&self, name: &str, model: Arc<dyn Model>);

    fn assign_input(&self, input_name: &str, transmitter: Box<dyn Input>);
    fn assign_output(&self, output_name: &str, transmitter: Box<dyn Output>);

    /// Prepares the futures running the treatment on track `track_id`.
    ///
    /// `secrets_access` is given only to treatments whose descriptor declares it.
    fn prepare(
        &self,
        track_id: usize,
        world: Weak<dyn World>,
        secrets_access: Option<SecretsAccess>,
        start: Pin<Box<dyn Future<Output = ()> + Send + Sync>>,
        finish: Pin<Box<dyn Future<Output = ()> + Send + Sync>>,
    ) -> Vec<TrackFuture>;
}
