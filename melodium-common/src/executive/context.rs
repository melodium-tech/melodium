
use core::fmt::Debug;
use downcast_rs::{DowncastSync, impl_downcast};
use crate::executive::Value;

pub trait Context : Debug + DowncastSync + Send + Sync {
    fn get_value(&self, name: &str) -> Option<&Value>;
}
impl_downcast!(sync Context);
