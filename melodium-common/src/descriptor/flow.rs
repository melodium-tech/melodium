use core::fmt::{Display, Formatter, Result};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Flow {
    Block,
    Stream,
}

impl Display for Flow {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Flow::Block => {
                write!(f, "Block")
            }
            Flow::Stream => {
                write!(f, "Stream")
            }
        }
    }
}
