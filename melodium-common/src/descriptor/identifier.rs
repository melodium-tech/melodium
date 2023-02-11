use core::{
    convert::TryFrom,
    fmt::{Display, Formatter},
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Identifier {
    path: Vec<String>,
    name: String,
}

impl Identifier {
    pub fn new(path: Vec<String>, name: &str) -> Self {
        if path.is_empty() {
            panic!("Identifier path cannot be empty.")
        }

        Self {
            path,
            name: name.to_string(),
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
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut string = self.path.join("/");

        string = string + "::" + &self.name;

        write!(f, "{}", string)
    }
}

impl TryFrom<String> for Identifier {
    type Error = String;

    fn try_from(value: String) -> core::result::Result<Self, Self::Error> {
        let full = value.split("::").collect::<Vec<_>>();
        if full.len() == 2 {
            let path = full[0];
            let name = full[1];

            let path = path.split('/').map(|s| s.to_string()).collect::<Vec<_>>();
            if path.len() >= 1 {
                Ok(Self {
                    path,
                    name: name.to_string(),
                })
            } else {
                Err(value)
            }
        } else {
            Err(value)
        }
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
