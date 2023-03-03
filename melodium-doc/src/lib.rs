use itertools::Itertools;
use melodium_common::descriptor::{
    Collection, CollectionTree, Context, Entry, Flow, Function, Identifier, Input, Model, Output,
    Parameter, Treatment,
};
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

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
        self.make_areas()?;
        for id in self.collection.identifiers() {
            self.make_entry(self.collection.get(&id).unwrap())?;
        }

        Ok(())
    }

    fn write(&self, file: &str, content: &[u8]) -> Result<(), std::io::Error> {
        let mut path = self.output.clone();
        path.push(file);
        std::fs::create_dir_all(path.parent().unwrap())?;
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

            content.push_str(&format!(
                "{margin}- [ {name}]({}/index.md)\n",
                sub_path.join("/")
            ));
            content.push_str(&Self::summary_area(
                area.areas.get(name).as_ref().unwrap(),
                sub_path,
            ));
        }

        for entry in area.entries.iter().sorted() {
            let line = match entry {
                Entry::Context(c) => {
                    format!("- [ {}]({})\n", c.name(), Self::id_filepath(c.identifier()))
                }
                Entry::Function(f) => format!(
                    "- [ {}]({})\n",
                    f.identifier().name(),
                    Self::id_filepath(f.identifier())
                ),
                Entry::Model(m) => format!(
                    "- [⬢ {}]({})\n",
                    m.identifier().name(),
                    Self::id_filepath(m.identifier())
                ),
                Entry::Treatment(t) => format!(
                    "- [⤇ {}]({})\n",
                    t.identifier().name(),
                    Self::id_filepath(t.identifier())
                ),
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
        let is_root = path.is_empty();

        let title = if is_root {
            Self::get_title()
        } else {
            format!("Area {}", path.last().unwrap())
        };

        let mut subs = String::new();
        for (sub_name, sub_area) in &area.areas {
            if subs.is_empty() {
                if is_root {
                    subs.push_str("## Packages\n\n");
                } else {
                    subs.push_str("## Subareas\n\n");
                }
            }

            subs.push_str(&format!("[{sub_name}]({sub_name}/index.md)  \n"));

            let mut sub_path = path.clone();
            sub_path.push(sub_name.clone());
            self.make_area(sub_area, sub_path)?;
        }

        let mut contexts = String::new();
        let mut functions = String::new();
        let mut models = String::new();
        let mut treatments = String::new();

        for entry in &area.entries {
            match entry {
                Entry::Context(c) => {
                    if contexts.is_empty() {
                        contexts.push_str("## Contexts\n\n");
                    }

                    contexts.push_str(&format!("[ {name}]({name}.md)  \n", name = c.name()));
                }
                Entry::Function(f) => {
                    if functions.is_empty() {
                        functions.push_str("## Functions\n\n");
                    }

                    functions.push_str(&format!(
                        "[ {name}]({name}.md)  \n",
                        name = f.identifier().name()
                    ));
                }
                Entry::Model(m) => {
                    if models.is_empty() {
                        models.push_str("## Models\n\n");
                    }

                    models.push_str(&format!(
                        "⬢[ {name}]({name}.md)  \n",
                        name = m.identifier().name()
                    ));
                }
                Entry::Treatment(t) => {
                    if treatments.is_empty() {
                        treatments.push_str("## Treatments\n\n");
                    }

                    treatments.push_str(&format!(
                        "⤇[ {name}]({name}.md)  \n",
                        name = t.identifier().name()
                    ));
                }
            }
        }

        let display_path = if !path.is_empty() {
            format!("\n\n`{}`", path.join("/"))
        } else {
            "".to_string()
        };

        let file = if is_root {
            "src/README.md".to_string()
        } else {
            format!("src/{}/index.md", path.join("/"))
        };
        let content = format!(
            "# {title}{display_path}\n\n---\n\n{subs}{contexts}{functions}{models}{treatments}"
        );

        self.write(&file, content.as_bytes())?;

        Ok(())
    }

    fn make_entry(&self, entry: &Entry) -> Result<(), Box<dyn Error>> {
        let content = match entry {
            Entry::Context(c) => self.context_content(c),
            Entry::Function(f) => self.function_content(f),
            Entry::Model(m) => self.model_content(m),
            Entry::Treatment(t) => self.treatment_content(t),
        };

        let file = format!(
            "src/{path}/{name}.md",
            path = entry.identifier().path().join("/"),
            name = entry.identifier().name()
        );

        self.write(&file, content.as_bytes())?;

        Ok(())
    }

    fn context_content(&self, context: &Arc<dyn Context>) -> String {
        let entries = if !context.values().is_empty() {
            let mut string = String::new();

            for entry_name in context.values().keys().sorted() {
                string.push_str(&format!(
                    "↪ `{}: {}`  \n",
                    entry_name,
                    context.values().get(entry_name).unwrap()
                ));
            }

            format!("#### Entries\n\n{}", string)
        } else {
            String::default()
        };

        format!(
            "# Context {name}\n\n`{id}`\n\n---\n\n{entries}\n\n---\n\n{doc}",
            name = context.identifier().name(),
            id = context.identifier().to_string(),
            doc = context.documentation(),
        )
    }

    fn function_content(&self, function: &Arc<dyn Function>) -> String {
        let parameters = if !function.parameters().is_empty() {
            let mut string = String::new();

            for param in function.parameters().iter() {
                string.push_str(&format!("↳ `{}: {}`  \n", param.name(), param.datatype()));
            }

            format!("#### Parameters\n\n{}", string)
        } else {
            String::default()
        };

        let call = format!(
            "{name}({params})",
            name = function.identifier().name(),
            params = function
                .parameters()
                .iter()
                .map(|p| p.name())
                .collect::<Vec<&str>>()
                .join(", ")
        );

        format!("# Function {name}\n\n`{id}`\n\n---\n\n#### Usage\n```\n{call}\n```\n\n{parameters}\n\n#### Return\n\n↴ `{return}`\n\n---\n\n{doc}",
            name = function.identifier().name(),
            id = function.identifier().to_string(),
            call = call,
            return = function.return_type(),
            parameters = parameters,
            doc = function.documentation(),
        )
    }

    fn model_content(&self, model: &Arc<dyn Model>) -> String {
        let parameters = if !model.parameters().is_empty() {
            let mut string = String::new();

            for param_name in model.parameters().keys().sorted() {
                string.push_str(&format!(
                    "↳ `{}`  \n",
                    Self::parameter(model.parameters().get(param_name).unwrap())
                ));
            }

            format!("#### Parameters\n\n{}", string)
        } else {
            String::default()
        };

        let base = if let Some(base_model) = model.base_model() {
            format!("Based on `{}`\n\n", base_model.identifier())
        } else {
            String::new()
        };

        format!(
            "# Model {name}\n\n`{id}`\n\n{base}---\n\n{parameters}\n\n---\n\n{doc}",
            name = model.identifier().name(),
            id = model.identifier().to_string(),
            base = base,
            parameters = parameters,
            doc = model.documentation(),
        )
    }

    fn treatment_content(&self, treatment: &Arc<dyn Treatment>) -> String {
        let models = if !treatment.models().is_empty() {
            let mut string = String::new();

            for name in treatment.models().keys().sorted() {
                /*string.push_str(&format!("⬡ `{name}: `{location}  \n",
                    location = self.get_location(descriptor.identifier(), model.identifier())
                ));*/
                string.push_str(&format!("⬡ `{name}: {type}`  \n",
                    type = treatment.models().get(name).unwrap().identifier()
                ));
            }

            format!("#### Configuration\n\n{}", string)
        } else {
            String::default()
        };

        let parameters = if !treatment.parameters().is_empty() {
            let mut string = String::new();

            for param_name in treatment.parameters().keys().sorted() {
                string.push_str(&format!(
                    "↳ `{}`\n",
                    Self::parameter(treatment.parameters().get(param_name).unwrap())
                ));
            }

            format!("#### Parameters\n\n{}", string)
        } else {
            String::default()
        };

        let requirements = if !treatment.contexts().is_empty() {
            let mut string = String::new();

            for name in treatment.contexts().keys().sorted() {
                string.push_str(&format!("○ `{}`  \n", name));
            }

            format!("#### Required contexts\n\n{}", string)
        } else {
            String::default()
        };

        let inputs = if !treatment.inputs().is_empty() {
            let mut string = String::new();

            for input_name in treatment.inputs().keys().sorted() {
                string.push_str(&format!(
                    "⇥ `{}: {}`  \n",
                    input_name,
                    Self::input(treatment.inputs().get(input_name).unwrap())
                ));
            }

            format!("#### Inputs\n\n{}", string)
        } else {
            String::default()
        };

        let outputs = if !treatment.outputs().is_empty() {
            let mut string = String::new();

            for output_name in treatment.outputs().keys().sorted() {
                string.push_str(&format!(
                    "↦ `{}: {}`  \n",
                    output_name,
                    Self::output(treatment.outputs().get(output_name).unwrap())
                ));
            }

            format!("#### Outputs\n\n{}", string)
        } else {
            String::default()
        };

        format!("# Treatment {name}\n\n`{id}`\n\n---\n\n{models}{parameters}{requirements}{inputs}{outputs}\n\n---\n\n{doc}",
            name = treatment.identifier().name(),
            id = treatment.identifier().to_string(),
            doc = treatment.documentation(),
        )
    }

    fn id_filepath(id: &Identifier) -> String {
        format!("{}/{}.md", id.path().join("/"), id.name())
    }

    fn parameter(parameter: &Parameter) -> String {
        format!("{var} {name}: {type}{val}",
            var = parameter.variability(),
            name = parameter.name(),
            type = parameter.datatype(),
            val = parameter.default().as_ref().map(|v| format!(" = {v}")).unwrap_or_default(),
        )
    }

    fn input(input: &Input) -> String {
        let flow = match input.flow() {
            Flow::Block => "Block",
            Flow::Stream => "Stream",
        };

        format!("{}<{}>", flow, input.datatype())
    }

    fn output(output: &Output) -> String {
        let flow = match output.flow() {
            Flow::Block => "Block",
            Flow::Stream => "Stream",
        };

        format!("{}<{}>", flow, output.datatype())
    }

    fn get_title() -> String {
        std::env::var("MELODIUM_DOC_TITLE").unwrap_or("Documentation".to_string())
    }

    fn get_author() -> String {
        std::env::var("MELODIUM_DOC_AUTHOR").unwrap_or("The Author".to_string())
    }

    fn default_mdbook_config() -> String {
        let title = Self::get_title();
        let author = Self::get_author();

        format!(
            r#"[book]
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
"#,
            author, title
        )
    }
}
