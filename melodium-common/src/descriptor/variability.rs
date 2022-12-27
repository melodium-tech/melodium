
use core::fmt::{Display, Formatter, Result};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Variability {
    Const,
    Var,
}

impl Display for Variability {

    fn fmt(&self, f: &mut Formatter<'_>) -> Result {

        write!(f, "{}", match self {
            Variability::Const => "const",
            Variability::Var => "var",
        })
    }
}
