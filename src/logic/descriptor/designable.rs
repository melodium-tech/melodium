
use super::super::builder::Builder;

pub trait Designable {
    fn register_builder(&self, builder: dyn Builder);
}
