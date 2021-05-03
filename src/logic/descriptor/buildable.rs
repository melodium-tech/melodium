
use std::sync::Arc;
use super::super::builder::Builder;

pub trait Buildable {

    fn builder(&self) -> Arc<Box<dyn Builder>>;
}
