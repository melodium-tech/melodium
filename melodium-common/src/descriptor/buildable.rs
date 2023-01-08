use crate::executive::{Model, Treatment, World};
use crate::descriptor::Treatment as TreatmentDescriptor;
use core::fmt::Debug;
use downcast_rs::{impl_downcast, DowncastSync};
use std::sync::{Arc, Weak};

pub trait Buildable<T>: Debug + DowncastSync + Send + Sync {
    fn build_mode(&self) -> T;
}
impl_downcast!(sync Buildable<T>);

pub enum ModelBuildMode {
    Compiled(fn(Arc<dyn World>) -> Arc<dyn Model>),
    Designed(),
}

pub enum TreatmentBuildMode {
    Compiled(fn() -> Arc<dyn Treatment>, Weak<dyn TreatmentDescriptor>),
    Source(Weak<dyn TreatmentDescriptor>),
    Designed(),
}
