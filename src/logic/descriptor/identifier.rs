
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

impl ToString for Identifier {
    
    fn to_string(&self) -> String {

        let mut string = match self.root {
            Root::Core => "core",
            Root::Std => "std",
            Root::Main => "main",
        }.to_string();

        for step in &self.path {
            string = string + "/" + &step;
        }

        string = string + "::" + &self.name;

        string
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Root {
    Core,
    Std,
    Main
}
