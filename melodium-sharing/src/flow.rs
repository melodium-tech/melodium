use melodium_common::descriptor::Flow as CommonFlow;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Flow {
    Block,
    Stream,
}

impl From<&CommonFlow> for Flow {
    fn from(value: &CommonFlow) -> Self {
        match value {
            CommonFlow::Block => Flow::Block,
            CommonFlow::Stream => Flow::Stream,
        }
    }
}

impl Into<CommonFlow> for Flow {
    fn into(self) -> CommonFlow {
        Into::into(&self)
    }
}

impl Into<CommonFlow> for &Flow {
    fn into(self) -> CommonFlow {
        match self {
            Flow::Block => CommonFlow::Block,
            Flow::Stream => CommonFlow::Stream,
        }
    }
}
