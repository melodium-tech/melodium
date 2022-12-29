
use core::fmt::{Debug};
use melodium_common::descriptor::Designer;

#[derive(Debug)]
pub struct Model {}

impl Designer for Model {
    fn set_collection(&self, collection: std::sync::Arc<melodium_common::descriptor::Collection>) {
        todo!()
    }
}
