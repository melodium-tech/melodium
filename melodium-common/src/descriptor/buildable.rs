use super::Designer;
use crate::executive::{Model, Treatment, World};
use core::fmt::Debug;
use downcast_rs::{impl_downcast, DowncastSync};
use std::sync::{Arc, RwLock};

pub trait Buildable<T>: Debug + DowncastSync + Send + Sync {
    fn build_mode(&self) -> &T;
}
impl_downcast!(sync Buildable<T>);

pub enum ModelBuildMode {
    Compiled(fn(Arc<dyn World>) -> Arc<dyn Model>),
    Designed(Arc<RwLock<dyn Designer>>),
}

pub enum TreatmentBuildMode {
    Compiled(fn() -> Arc<dyn Treatment>),
    Source,
    Designed(Arc<RwLock<dyn Designer>>),
}
