
use std::io::Write;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::{HashMap, hash_map::Entry};
use std::path::PathBuf;
use glob::glob;
use itertools::Itertools;
use crate::script::file::File;
use crate::script::path::{Path, PathRoot};
use crate::script::error::ScriptError;
use super::markdown;

pub struct Instance {
    pub root: PathRoot,
    pub entry_path: PathBuf,
    pub output_path: PathBuf,
    /// Files used in the instance.
    pub script_files: Vec<Rc<File>>,
}

impl Instance {

    pub fn new(root: PathRoot, entry_path: PathBuf, output_path: PathBuf) -> Self {
        Self {
            root,
            entry_path,
            output_path,
            script_files: Vec::new(),
        }
    }

    pub fn parse_files(&mut self) -> Result<(), (Vec<std::io::Error>, HashMap<PathBuf, ScriptError>)> {

        let mut io_errors = Vec::new();
        let mut script_errors = HashMap::new();

        for entry in glob(&format!("{}/**/*.mel", self.entry_path.to_str().unwrap())).unwrap() {
            match entry {
                Ok(entry) => {

                    let absolute_path;
                    match entry.canonicalize() {
                        Ok(ap) => absolute_path = ap,
                        Err(e) => {
                            io_errors.push(e);
                            continue;
                        },
                    };
                    let relative_path = absolute_path.strip_prefix(&self.entry_path).unwrap();
                    let mut path_steps: Vec<&str> = relative_path.to_str().unwrap().strip_suffix(".mel").unwrap().split('/').collect();
                    path_steps.insert(0, match self.root {
                        PathRoot::Main => "main",
                        PathRoot::Std => "std",
                        _ => "",
                    });
                    let path = Path::new(path_steps.iter().map(|s| s.to_string()).collect());


                    let mut file = File::new(path, absolute_path);

                    if let Err(e) = file.read() {
                        io_errors.push(e);
                        continue;
                    }

                    if let Err(e) = file.parse() {
                        script_errors.insert(file.absolute_path, e);
                        continue;
                    }

                    self.script_files.push(Rc::new(file));
                },
                Err(e) => {
                    io_errors.push(e.into_error());
                },
            }
        }

        if io_errors.is_empty() && script_errors.is_empty() {
            Ok(())
        }
        else {
            Err((io_errors, script_errors))
        }
    }

    pub fn output_doc(&self) -> std::io::Result<()> {

        std::fs::create_dir_all(&self.output_path)?;

        std::fs::write(self.output_path.join("book.toml"), Self::default_mdbook_config())?;

        for script in &self.script_files {

            let output_path = self.get_output_path(&script.path);
            std::fs::create_dir_all(output_path.clone())?;
            let mut file = std::fs::File::create(output_path.join("README.md"))?;

            let mut content = String::new();
            
            content.push_str("## Models\n");
            for (_, model) in &script.semantic.as_ref().unwrap().script.read().unwrap().models {

                let model = model.read().unwrap();

                content.push_str(&format!("- [{}]({}.md)\n", model.name, model.name));

                std::fs::write(output_path.join(format!("{}.md", model.name)), markdown::model(&model, &script.path).as_bytes())?;
            }

            content.push_str("## Sequences\n");
            for (_, sequence) in &script.semantic.as_ref().unwrap().script.read().unwrap().sequences {

                let sequence = sequence.read().unwrap();

                content.push_str(&format!("- [{}]({}.md)\n", sequence.name, sequence.name));

                std::fs::write(output_path.join(format!("{}.md", sequence.name)), markdown::sequence(&sequence, &script.path).as_bytes())?;
            }

            file.write_all(content.as_bytes())?;
        }

        std::fs::write(self.output_path.join("src/SUMMARY.md"), self.generate_summary())?;

        Ok(())
    }

    fn generate_summary(&self) -> String {

        struct Node {
            files: RefCell<HashMap<String, (Option<Rc<File>>, Rc<Node>)>>
        }
        let hierarchy = Rc::new(Node { files: RefCell::new(HashMap::new())});

        for file in &self.script_files {

            let mut parent_node = Rc::clone(&hierarchy);
            let mut level = 0;
            let mut last_entry = file.path.path().get(level).unwrap();
            while let Some(next_entry) = file.path.path().get(level + 1) {

                // We know last_entry is not its name, so
                // we want to get the sub named 'last_entry'
                let next_parent;
                match parent_node.files.borrow_mut().entry(last_entry.clone()) {
                    Entry::Occupied(entry) => next_parent = Rc::clone(&entry.get().1),
                    Entry::Vacant(entry) => next_parent = Rc::clone(&entry.insert((None, Rc::new(Node { files: RefCell::new(HashMap::new()), /*subs: RefCell::new(HashMap::new()) */ }))).1),
                }

                parent_node = next_parent;
                last_entry = next_entry;
                level += 1;
            }

            // last_entry is the file name
            match parent_node.files.borrow_mut().entry(last_entry.clone()) {
                Entry::Occupied(mut entry) => entry.get_mut().0 = Some(Rc::clone(file)),
                Entry::Vacant(entry) => { entry.insert((Some(Rc::clone(file)), Rc::new(Node { files: RefCell::new(HashMap::new())}))); },
            };
        }

        fn make_node(level: usize, node: Rc<Node>, path: String) -> String {
            let mut string = String::new();

            for file_name in node.files.borrow().keys().sorted() {
                let file = &node.files.borrow()[file_name].0;

                if file.is_some() {
                    (0..level).for_each(|_| string.push_str("  "));
                    string.push_str("- ");

                    string.push_str(&format!("[{}]({}{}/README.md)\n", file_name, path, file_name));
                }
                else {
                    (0..level).for_each(|_| string.push_str("  "));
                    string.push_str("- ");
                    string.push_str(&format!("[{}]()\n", file_name));
                }

                let next_path = format!("{}{}/", path, file_name);
    
                string.push_str(&make_node(level + 1, Rc::clone(&node.files.borrow()[file_name].1), next_path));
                
                if let Some(file) = file {
                    for (_, model) in &file.semantic.as_ref().unwrap().script.read().unwrap().models {

                        let model = model.read().unwrap();
        
                        (0..=level).for_each(|_| string.push_str("  "));
                        string.push_str(&format!("- [{}]({}{}/{}.md)\n", model.name, path, file_name, model.name));
                    }
    
                    for (_, sequence) in &file.semantic.as_ref().unwrap().script.read().unwrap().sequences {
    
                        let sequence = sequence.read().unwrap();
        
                        (0..=level).for_each(|_| string.push_str("  "));
                        string.push_str(&format!("- [{}]({}{}/{}.md)\n", sequence.name, path, file_name, sequence.name));
                    }
                }              
            }


            string
        }

        let mut output = String::from("# Summary\n\n");
        output.push_str(&make_node(0, hierarchy, "".to_string()));

        output
    }

    fn get_output_path(&self, path: &Path) -> PathBuf {

        let mut os_path = self.output_path.join("src");

        for step in path.path() {
            os_path = os_path.join(step);
        }

        os_path
    }

    fn default_mdbook_config() -> &'static str {
        r#"
        [book]
        authors = ["The Author"]
        language = "en"
        multilingual = false
        src = "src"
        title = "Documentation"

        [output.html]
        no-section-label = true

        [output.html.fold]
        enable = true
        level = 0 
        "#
    }
}