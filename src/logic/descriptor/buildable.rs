
use super::super::builder::Builder;

pub trait Buildable {

    fn builder(&self) -> &dyn Builder;
}
