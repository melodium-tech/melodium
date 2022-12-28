use crate::executive::Value;
use core::fmt::Debug;
use downcast_rs::{impl_downcast, DowncastSync};

pub trait Context: Debug + DowncastSync + Send + Sync {
    fn get_value(&self, name: &str) -> Option<&Value>;
}
impl_downcast!(sync Context);
