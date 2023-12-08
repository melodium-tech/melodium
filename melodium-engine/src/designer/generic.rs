use std::collections::HashMap;
use std::fmt::Debug;
use melodium_common::descriptor::DescribedType;

pub trait Generic: Send + Sync + Debug {
    fn generics(&self) -> &HashMap<String, DescribedType>;
}
