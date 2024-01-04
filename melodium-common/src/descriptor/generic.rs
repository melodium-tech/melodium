use core::fmt::{Debug, Display};
use downcast_rs::{impl_downcast, DowncastSync};

pub trait Generic: DowncastSync + Display + Debug + Send + Sync {
    fn generics(&self) -> &Vec<String>;
}
impl_downcast!(sync Generic);
