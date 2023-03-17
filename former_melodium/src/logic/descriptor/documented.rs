
use std::fmt::{Debug, Display};
use downcast_rs::{DowncastSync, impl_downcast};
pub trait Documented: DowncastSync + Display + Debug + Send + Sync {
    fn documentation(&self) -> &str;
}
impl_downcast!(sync Documented);
