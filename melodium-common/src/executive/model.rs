use crate::descriptor::Model as ModelDescriptor;
use crate::executive::{SecretsAccess, Value};
use core::fmt::Debug;
use downcast_rs::{impl_downcast, DowncastSync};
use std::{collections::HashMap, sync::Arc};

pub type ModelId = usize;

pub trait Model: Debug + DowncastSync + Send + Sync {
    fn descriptor(&self) -> Arc<dyn ModelDescriptor>;

    fn id(&self) -> Option<ModelId>;
    fn set_id(&self, id: ModelId);

    fn set_parameter(&self, param: &str, value: Value);

    /// Gives the model its access to secrets, if its descriptor declares it.
    fn set_secrets_access(&self, access: SecretsAccess);
    /// Registers the secret sources of the model, before any model is initialized.
    fn register_secret_sources(&self);

    fn initialize(&self);
    fn shutdown(&self);

    fn invoke_source(&self, source: &str, params: HashMap<String, Value>);
}
impl_downcast!(sync Model);
