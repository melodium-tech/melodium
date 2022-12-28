use core::fmt::{Debug, Display};
use downcast_rs::{impl_downcast, DowncastSync};

pub trait Documented: DowncastSync + Display + Debug + Send + Sync {
    fn documentation(&self) -> &str;
}
impl_downcast!(sync Documented);
