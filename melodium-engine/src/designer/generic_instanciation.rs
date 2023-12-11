use crate::LogicResult;
use melodium_common::descriptor::DescribedType;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait GenericInstanciation: Send + Sync + Debug {
    fn set_generic(&mut self, generic: String, r#type: DescribedType) -> LogicResult<()>;
    fn generics(&self) -> &HashMap<String, DescribedType>;
}
