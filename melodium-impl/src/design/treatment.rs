
use core::fmt::{Debug};
use melodium_common::descriptor::Designer;

#[derive(Debug)]
pub struct Treatment {}

impl Designer for Treatment {
    fn set_collection(&self, collection: std::sync::Arc<melodium_common::descriptor::Collection>) {
        todo!()
    }
}
