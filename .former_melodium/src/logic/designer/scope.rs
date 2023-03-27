
use std::fmt::Debug;
use std::sync::Arc;
use super::super::collection_pool::CollectionPool;
use super::super::descriptor::ParameterizedDescriptor;

pub trait Scope : Send + Sync + Debug {

    fn descriptor(&self) -> Arc<dyn ParameterizedDescriptor>;
    fn collections(&self) -> Arc<CollectionPool>;
}