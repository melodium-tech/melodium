
macro_rules! core_identifier {
    ($($step:expr),*;$name:expr) => {
        crate::logic::descriptor::identifier::Identifier::new(
            crate::logic::descriptor::identifier::Root::Core,
            {
                let mut path=Vec::new();
                $(path.push($step.to_string());)*
                path
            },
            $name
        )
    };
}
pub(crate) use core_identifier;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Identifier {
    root: Root,
    path: Vec<String>,
    name: String
}

impl Identifier {
    pub fn new (root: Root, path: Vec<String>, name: &str) -> Self {
        Self {
            root,
            path,
            name: name.to_string()
        }
    }

    pub fn root(&self) -> &Root {
        &self.root
    }

    pub fn path(&self) -> &Vec<String> {
        &self.path
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let mut string = self.root.to_string();

        for step in &self.path {
            string = string + "/" + &step;
        }

        string = string + "::" + &self.name;

        write!(f, "{}", string)
    }
}

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.to_string().partial_cmp(&other.to_string())
    }
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Root {
    Core,
    Std,
    Main
}

impl std::fmt::Display for Root {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        write!(f, "{}", match self {
            Root::Core => "core",
            Root::Main => "main",
            Root::Std  => "std",
        })
    }
}
