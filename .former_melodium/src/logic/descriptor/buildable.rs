
use std::fmt::Debug;
use std::sync::Arc;
use super::super::builder::Builder;

pub trait Buildable : Debug + Send + Sync {

    fn builder(&self) -> Arc<Box<dyn Builder>>;
}
