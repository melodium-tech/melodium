
#[derive(Clone, PartialEq, Eq, Hash)]
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

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Root {
    Core,
    Std,
    Main
}
