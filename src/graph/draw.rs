
use std::sync::{Arc, RwLock};
use crate::logic::designer::*;

pub fn draw(sequence: Arc<RwLock<SequenceDesigner>>) -> String {

    let sequence = sequence.read().unwrap();
    let mut result = String::new();

    let mut max_level = 0;
    for (_, t) in sequence.treatments() {
        
        let level = t.read().unwrap().level();

        if max_level < level {
            max_level = level;
        }
    }

    result.push_str(&format!(r#"<svg height="30" width="{}">"#, (max_level + 1) * 10));

    for (name, treatment) in sequence.treatments() {
        result.push_str(&format!(r#"<text x="{}" y="15">{}</text>"#, treatment.read().unwrap().level() * 10, name));
    }

    result.push_str(&format!("</svg>"));

    result
}

