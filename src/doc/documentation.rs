
use std::sync::Arc;
use std::path::PathBuf;
use itertools::Itertools;
use crate::logic::collection_pool::CollectionPool;
use crate::logic::descriptor::identifier::Identifier;
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

    pub fn summary(&self) -> String {

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

        println!("Current path: {}", path.join("/"));
        // The '-1' in sub_areas.push(id.path().get(path.len()-1).unwrap().clone()) could be removed once root is included in path
        // also true_path could be removed

        let mut functions = String::new();
        for id in self.collection.functions.get_tree_path(&path).iter().sorted() {
            
            if Self::true_path(id) == path {
                (0..=level).for_each(|_| functions.push_str("  "));
                functions.push_str(&format!("- [ {func}]({path}/{func}.md)\n",
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
                println!("Path length: {}, Id: {}", path.len(), id);
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

    fn area(&self, path: Vec<String>) -> String {
        todo!()
    }
}
