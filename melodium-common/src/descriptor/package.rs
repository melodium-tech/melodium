use crate::descriptor::{Collection, Loader, LoadingResult, Version};
use core::fmt::Debug;
use std::collections::HashMap;

pub trait Package: Debug {
    fn name(&self) -> &str;
    fn version(&self) -> &Version;
    fn requirements(&self) -> &Vec<&str>;
    fn collection(&self, loader: &dyn Loader) -> LoadingResult<Collection>;
    fn embedded(&self) -> &HashMap<&'static str, &'static [u8]>;
}
