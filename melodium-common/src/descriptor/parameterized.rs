
use core::fmt::Debug;
use std::collections::HashMap;
use std::sync::Arc;
use super::Parameter;

pub trait Parameterized : Debug + Send + Sync {
    fn parameters(&self) -> &HashMap<String, Parameter>;
    fn as_parameterized(&self) -> Arc<dyn Parameterized>;
}
