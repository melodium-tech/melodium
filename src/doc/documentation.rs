
use std::sync::Arc;
use std::path::PathBuf;
use clap::crate_version;
use indoc::formatdoc;
use itertools::Itertools;
use crate::logic::collection_pool::CollectionPool;
use crate::logic::descriptor::identifier::Identifier;
use crate::logic::descriptor::*;
pub struct Documentation {
    pub roots: Vec<String>,
    pub collection: Arc<CollectionPool>,
    pub output_path: PathBuf,
}

impl Documentation {

    pub fn new(roots: Vec<String>, collection: Arc<CollectionPool>, output_path: PathBuf) -> Self {
        Self {
            roots,
            collection,
            output_path,
        }
    }

    fn true_path(id: &Identifier) -> Vec<String> {
        let mut path = vec![id.root().to_string()];
        path.extend(id.path().clone());
        path
    }

    pub fn make(&self) -> std::io::Result<()> {

        let path = self.output_path.join("src");
        std::fs::create_dir_all(&path)?;
        
        let areas = self.areas();

        for area in areas {

            let mut path = path.clone();
            path.push(area.join("/"));
            std::fs::create_dir_all(&path)?;

            path.push("README.md");

            let contents = self.area(area);
            
            std::fs::write(path, contents)?;
        }

        std::fs::write(path.join("SUMMARY.md"), self.summary())?;
        std::fs::write(self.output_path.join("book.toml"), Self::default_mdbook_config())?;

        for id in self.collection.functions.identifiers() {
            self.write_element(&id, self.function(self.collection.functions.get(&id).unwrap()))?;
        }

        for id in self.collection.models.identifiers() {
            self.write_element(&id, self.model(self.collection.models.get(&id).unwrap()))?;
        }

        for id in self.collection.treatments.identifiers() {
            self.write_element(&id, self.treatment(self.collection.treatments.get(&id).unwrap()))?;
        }

        Ok(())
    }

    fn summary(&self) -> String {

        let mut output = String::new();

        output.push_str("# Summary\n\n[Documentation](README.md)\n");

        for root in self.roots.iter().sorted() {
            output.push_str(&self.summary_area(vec![root.clone()]));
        }

        output
    }

    fn summary_area(&self, path: Vec<String>) -> String {
        let level = path.len() - 1;
        let mut sub_areas = Vec::new();

        // The '-1' in sub_areas.push(id.path().get(path.len()-1).unwrap().clone()) could be removed once root is included in path
        // also true_path could be removed

        let mut functions = String::new();
        for id in self.collection.functions.get_tree_path(&path).iter().sorted() {
            
            if Self::true_path(id) == path {
                (0..=level).for_each(|_| functions.push_str("  "));
                functions.push_str(&format!("- [ {func}]({path}/{func}.md)\n",
                    func = id.name(),
                    path = path.join("/"),
                ));
            }
            else {
                sub_areas.push(id.path().get(path.len()-1).unwrap().clone())
            }
        }

        let mut models = String::new();
        for id in self.collection.models.get_tree_path(&path).iter().sorted() {
            
            if Self::true_path(id) == path {
                (0..=level).for_each(|_| models.push_str("  "));
                models.push_str(&format!("- [⬢ {model}]({path}/{model}.md)\n",
                    model = id.name(),
                    path = path.join("/"),
                ));
            }
            else {
                sub_areas.push(id.path().get(path.len()-1).unwrap().clone())
            }
        }

        let mut treatments = String::new();
        for id in self.collection.treatments.get_tree_path(&path).iter().sorted() {
            
            if Self::true_path(id) == path {
                (0..=level).for_each(|_| treatments.push_str("  "));
                treatments.push_str(&format!("- [⤇ {treatment}]({path}/{treatment}.md)\n",
                    treatment = id.name(),
                    path = path.join("/"),
                ));
            }
            else {
                sub_areas.push(id.path().get(path.len()-1).unwrap().clone())
            }
        }

        let mut subs = String::new();
        let sub_areas: Vec<String>  = sub_areas.iter().unique().map(|s| s.clone()).collect();
        for sub in sub_areas {
            let mut path = path.clone();
            path.push(sub);

            subs.push_str(&self.summary_area(path));
        }


        let mut marging = String::new();
        (0..level).for_each(|_| marging.push_str("  "));

        format!("{marging}- [{area}]({path}/README.md)\n{}{}{}{}", subs, functions, models, treatments,
            area = path.get(path.len()-1).unwrap(),
            path = path.join("/"),
        )
    }

    fn write_element(&self, id: &Identifier, contents: String) -> std::io::Result<()> {

        let mut path = self.output_path.join("src");
        path.push(Self::true_path(id).join("/"));

        std::fs::create_dir_all(&path)?;

        path.push(&format!("{}.md", id.name()));

        std::fs::write(path, contents)
    }

    fn areas(&self) -> Vec<Vec<String>> {

        // Known to be extremely naïve implementation

        let mut areas = Vec::new();

        self.collection.functions.identifiers().iter().for_each(|id| areas.push(Self::true_path(id)));
        self.collection.models.identifiers().iter().for_each(|id| areas.push(Self::true_path(id)));
        self.collection.treatments.identifiers().iter().for_each(|id| areas.push(Self::true_path(id)));

        areas.iter().flat_map(|area| {
            let mut steps = Vec::new();
            let mut parents = Vec::new();
            for i in 0..area.len() {
                steps.push(area.get(i).unwrap().clone());
                parents.push(steps.clone());
            }
            parents
        }).unique().collect()
    }

    fn area(&self, path: Vec<String>) -> String {
        let mut sub_areas = Vec::new();

        // The '-1' in sub_areas.push(id.path().get(path.len()-1).unwrap().clone()) could be removed once root is included in path
        // also true_path could be removed

        let mut functions = String::new();
        for id in self.collection.functions.get_tree_path(&path).iter().sorted() {
            
            if Self::true_path(id) == path {
                functions.push_str(&format!("[ {func}]({func}.md)  \n",
                    func = id.name(),
                ));
            }
            else {
                sub_areas.push(id.path().get(path.len()-1).unwrap().clone())
            }
        }
        if !functions.is_empty() {
            functions = format!("## Functions\n\n{}", functions);
        }

        let mut models = String::new();
        for id in self.collection.models.get_tree_path(&path).iter().sorted() {
            
            if Self::true_path(id) == path {
                models.push_str(&format!("⬢ [{model}]({model}.md)  \n",
                    model = id.name(),
                ));
            }
            else {
                sub_areas.push(id.path().get(path.len()-1).unwrap().clone())
            }
        }
        if !models.is_empty() {
            models = format!("## Models\n\n{}", models);
        }

        let mut treatments = String::new();
        for id in self.collection.treatments.get_tree_path(&path).iter().sorted() {
            
            if Self::true_path(id) == path {
                treatments.push_str(&format!("⤇ [{treatment}]({treatment}.md)  \n",
                    treatment = id.name(),
                ));
            }
            else {
                sub_areas.push(id.path().get(path.len()-1).unwrap().clone())
            }
        }
        if !treatments.is_empty() {
            treatments = format!("## Treatments\n\n{}", treatments);
        }

        let mut subs = String::new();
        let sub_areas: Vec<String>  = sub_areas.iter().unique().map(|s| s.clone()).collect();
        for area in sub_areas {
            
            subs.push_str(&format!("[{area}]({area}/index.md)  \n"));
        }
        if !subs.is_empty() {
            subs = format!("## Subareas\n\n{}", subs);
        }

        format!("# Area {area}\n\n`{path}`\n\n---\n\n{subs}{functions}{models}{treatments}",
            area = path.get(path.len()-1).unwrap(),
            path = path.join("/"),
        )
    }

    fn get_location(&self, local: &Identifier, to: &Identifier) -> String {

        let local_path = Self::true_path(local);
        let to_path = Self::true_path(to);

        let mut url = String::new();

        if self.roots.contains(to_path.get(0).unwrap()) {

            (0..local_path.len()).for_each(|_| url.push_str("../"));
            url.push_str(&to_path.join("/"));
            url.push_str(&format!("/{}.md", to.name()));
        } else {

            url.push_str(&format!("https://doc.melodium.tech/{}/{}/{}.html",
                crate_version!(),
                to_path.join("/"),
                to.name(),
            ));
        }

        format!("[`{to}`]({url})")
    }

    fn function(&self, descriptor: &Arc<dyn FunctionDescriptor>) -> String {
        
        let parameters = if !descriptor.parameters().is_empty() {
            let mut string = String::new();

            for param in descriptor.parameters().iter() {
                string.push_str(&format!("↳ `{}: {}`  \n", param.name(), param.datatype()));
            }

            format!("#### Parameters\n\n{}", string)
        }
        else { String::default() };

        let call = format!("{name}({params})",
            name = descriptor.identifier().name(),
            params = descriptor.parameters().iter().map(|p| p.name()).collect::<Vec<&str>>().join(", ")
        );

        format!("# Function {name}\n\n`{id}`\n\n---\n\n#### Usage\n```\n{call}\n```\n\n{parameters}\n\n#### Return\n\n↴ `{return}`\n\n---\n\n{doc}",
            name = descriptor.identifier().name(),
            id = descriptor.identifier().to_string(),
            call = call,
            return = descriptor.return_type(),
            parameters = parameters,
            doc = descriptor.documentation(),
        )
    }

    fn model(&self, descriptor: &Arc<dyn ModelDescriptor>) -> String {
        
        let parameters = if !descriptor.parameters().is_empty() {
            let mut string = String::new();

            for (_, param) in descriptor.parameters().iter() {
                string.push_str(&format!("↳ `{}`\n", Self::parameter(&param)));
            }

            format!("#### Parameters\n\n{}", string)
        }
        else { String::default() };

        let base = if !descriptor.is_core_model() {
            format!("Based on {}\n\n", self.get_location(descriptor.identifier(), descriptor.core_model().identifier()))
        }
        else {
            String::new()
        };

        format!("# Model {name}\n\n`{id}`\n\n{base}---\n\n{parameters}\n\n---\n\n{doc}",
            name = descriptor.identifier().name(),
            id = descriptor.identifier().to_string(),
            base = base,
            parameters = parameters,
            doc = descriptor.documentation(),
        )
    }

    fn treatment(&self, descriptor: &Arc<dyn TreatmentDescriptor>) -> String {

        let models = if !descriptor.models().is_empty() {
            let mut string = String::new();

            for (name, model) in descriptor.models() {
                /*string.push_str(&format!("⬡ `{name}: `{location}  \n",
                    location = self.get_location(descriptor.identifier(), model.identifier())
                ));*/
                string.push_str(&format!("⬡ `{name}: {type}`  \n",
                    type = model.identifier()
                ));
            }

            format!("#### Configuration\n\n{}", string)
        }
        else { String::default() };
        
        let parameters = if !descriptor.parameters().is_empty() {
            let mut string = String::new();

            for (_, param) in descriptor.parameters().iter() {
                string.push_str(&format!("↳ `{}`\n", Self::parameter(&param)));
            }

            format!("#### Parameters\n\n{}", string)
        }
        else { String::default() };

        let requirements = if !descriptor.requirements().is_empty() {
            let mut string = String::new();
    
            for (_, req) in descriptor.requirements() {
                string.push_str(&format!("○ `{}`  \n", req.name()));
            }
    
            format!("#### Require\n\n{}", string)
        }
        else { String::default() };

        let inputs = if !descriptor.inputs().is_empty() {
            let mut string = String::new();
    
            for (_, input) in descriptor.inputs() {
                string.push_str(&format!("⇥ `{}: {}`  \n", input.name(), Self::input(input)));
            }
    
            format!("#### Inputs\n\n{}", string)
        }
        else { String::default() };

        let outputs = if !descriptor.outputs().is_empty() {
            let mut string = String::new();
    
            for (_, output) in descriptor.outputs() {
                string.push_str(&format!("↦ `{}: {}`  \n", output.name(), Self::output(output)));
            }
    
            format!("#### Outputs\n\n{}", string)
        }
        else { String::default() };

        format!("# Sequence {name}\n\n`{id}`\n\n---\n\n{models}{parameters}{requirements}{inputs}{outputs}\n\n---\n\n{doc}",
            name = descriptor.identifier().name(),
            id = descriptor.identifier().to_string(),
            doc = descriptor.documentation(),
        )
    }

    fn parameter(parameter: &ParameterDescriptor) -> String {

        format!("{var} {name}: {type}{val}",
            var = parameter.variability(),
            name = parameter.name(),
            type = parameter.datatype(),
            val = parameter.default().as_ref().map(|v| format!(" = {v}")).unwrap_or_default(),
        )
    }

    fn input(input: &InputDescriptor) -> String {

        let flow = match input.flow() {
            FlowDescriptor::Block => "Block",
            FlowDescriptor::Stream => "Stream",
        };
    
        format!("{}<{}>", flow, input.datatype())
    }

    fn output(output: &OutputDescriptor) -> String {

        let flow = match output.flow() {
            FlowDescriptor::Block => "Block",
            FlowDescriptor::Stream => "Stream",
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

        let title  = Self::get_title();
        let author = Self::get_author();

        formatdoc!(r#"
        [book]
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
