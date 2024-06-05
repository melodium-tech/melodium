use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ImplementationKind<D: Serialize + Deserialize + Clone + Debug + PartialEq> {
    Compiled,
    Designed(D),
}
