use core::fmt::Debug;
use downcast_rs::{impl_downcast, DowncastSync};

pub trait Reference: Debug + DowncastSync + Send + Sync {}
impl_downcast!(sync Reference);
