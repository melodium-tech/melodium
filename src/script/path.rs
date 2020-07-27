
#[derive(Clone)]
pub struct Path {
    path: Vec<String>,
    root: PathRoot
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum PathRoot {
    Core,
    Std,
    Main,
    Local,
    Other,
}

impl Path {
    pub fn new(path: Vec<String>) -> Self {

        let first = path.first();
        let root = if let Some(val) = first {
            match val.as_str() {
                "core" => PathRoot::Core,
                "std" => PathRoot::Std,
                "main" => PathRoot::Main,
                "local" => PathRoot::Local,
                _ => PathRoot::Other
            }
        }
        else {
            PathRoot::Other
        };

        Self {
            path,
            root
        }
    }

    pub fn path(&self) -> &Vec<String> {
        &self.path
    }

    pub fn root(&self) -> PathRoot {
        self.root
    }

    pub fn is_valid(&self) -> bool {
        self.root != PathRoot::Other
    }
}
