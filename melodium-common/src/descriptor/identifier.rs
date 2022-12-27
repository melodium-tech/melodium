
use core::fmt::{Display, Formatter, Result};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Identifier {
    path: Vec<String>,
    name: String
}

impl Identifier {
    pub fn new(path: Vec<String>, name: &str) -> Self {

        if path.is_empty() {
            panic!("Identifier path cannot be empty.")
        }

        Self {
            path,
            name: name.to_string()
        }
    }

    pub fn root(&self) -> &String {
        &self.path.first().unwrap()
    }

    pub fn path(&self) -> &Vec<String> {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {

        let mut string = self.path.join("/");

        string = string + "::" + &self.name;

        write!(f, "{}", string)
    }
}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.to_string().partial_cmp(&other.to_string())
    }
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.to_string().cmp(&other.to_string())
    }
}
