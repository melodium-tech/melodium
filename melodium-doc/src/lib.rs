
use melodium_common::descriptor::{Collection, CollectionTree, Entry, Identifier};
use std::error::Error;
use std::path::PathBuf;
use itertools::Itertools;

#[derive(Clone, Debug)]
pub struct Documentation {
    collection: Collection,
    tree: CollectionTree,
    output: PathBuf,
}

impl Documentation {

    pub fn new(output: PathBuf, collection: Collection) -> Self {
        Self {
            tree: collection.get_tree(),
            collection,
            output,
        }
    }

    pub fn make_documentation(&self) -> Result<(), Box<dyn Error>> {

        self.write("book.toml", Self::default_mdbook_config().as_bytes())?;
        self.make_summary()?;

        Ok(())
    }

    fn write(&self, file: &str, content: &[u8]) -> Result<(), std::io::Error> {
        let mut path = self.output.clone();
        std::fs::create_dir_all(&path)?;
        path.push(file);
        std::fs::write(path, content)
    }

    fn make_summary(&self) -> Result<(), Box<dyn Error>> {

        let mut md = String::from("# Summary\n\n[Documentation](README.md)\n");

        md.push_str(&Self::summary_area(&self.tree, vec![]));

        self.write("src/SUMMARY.md", md.as_bytes())?;

        Ok(())
    }

    fn summary_area(area: &CollectionTree, path: Vec<String>) -> String {

        let mut content = String::new();

        let mut margin = String::new();
        (0..path.len()).for_each(|_| margin.push_str("  "));

        for name in area.areas.keys().sorted() {
            let mut sub_path = path.clone();
            sub_path.push(name.clone());

            content.push_str(&format!("{margin}- [ {name}]({}/index.md)\n", sub_path.join("/")));
            content.push_str(&Self::summary_area(area.areas.get(name).as_ref().unwrap(), sub_path));
        }

        for entry in area.entries.iter().sorted() {
            let line = match entry {
                Entry::Context(c) => format!("- [ {}]({})\n", c.name(), Self::id_filepath(c.identifier())),
                Entry::Function(f) => format!("- [ {}]({})\n", f.identifier().name(), Self::id_filepath(f.identifier())),
                Entry::Model(m) => format!("- [⬢ {}]({})\n", m.identifier().name(), Self::id_filepath(m.identifier())),
                Entry::Treatment(t) => format!("- [⤇ {}]({})\n", t.identifier().name(), Self::id_filepath(t.identifier())),
            };

            content.push_str(&margin);
            content.push_str(&line);
        }

        content
    }

    fn make_areas(&self) -> Result<(), Box<dyn Error>> {
        self.make_area(&self.tree, vec![])
    }

    fn make_area(&self, area: &CollectionTree, path: Vec<String>) -> Result<(), Box<dyn Error>> {

        let title = if path.is_empty() {
            Self::get_title()
        } else {
            format!("Area {}", path.last().unwrap())
        };

        let mut subs = String::new();
        for (sub_area, _) in &area.areas {
            if subs.is_empty() {
                subs.push_str("## Subareas\n\n");
            }

            subs.push_str(&format!("[{sub_area}]({sub_area}/index.md)  \n"));
        }

        let mut contexts = String::new();
        let mut functions = String::new();
        let mut models = String::new();
        let mut treatments = String::new();

        for entry in &area.entries {
            match entry {
                Entry::Context(c) => {},
                Entry::Function(_) => todo!(),
                Entry::Model(_) => todo!(),
                Entry::Treatment(_) => todo!(),
            }
        }

        let path = if !path.is_empty() {
            format!("\n\n`{}`", path.join("/"))
        } else { "".to_string() };

        format!("# {title}{path}\n\n---\n\n{subs}{contexts}{functions}{models}{treatments}");
        Ok(())
    }

    fn id_filepath(id: &Identifier) -> String {
        format!("{}/{}.md", id.path().join("/"), id.name())
    }

    fn get_title() -> String {
        std::env::var("MELODIUM_DOC_TITLE").unwrap_or("Documentation".to_string())
    }

    fn get_author() -> String {
        std::env::var("MELODIUM_DOC_AUTHOR").unwrap_or("The Author".to_string())
    }

    fn default_mdbook_config() -> String {

        let title  = Self::get_title();
        let author = Self::get_author();

        format!(r#"[book]
authors = ["{}"]
language = "en"
multilingual = false
src = "src"
title = "{}"

[output.html]
no-section-label = true

[output.html.fold]
enable = true
level = 0 

[output.html.print]
enable = false
"#, author, title)
    }

}

