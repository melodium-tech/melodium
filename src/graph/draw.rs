
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use itertools::Itertools;

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

    result.push_str(&format!(r#"<svg xmlns="http://www.w3.org/2000/svg" height="1200" width="{}"><style>{}</style>"#, (max_level + 2) * 400, include_str!("style.css")));

    let mut sizes: HashMap<String, (u64, u64)> = HashMap::new();
    for (_name, t) in sequence.treatments() {
        //result.push_str(&format!(r#"<text x="{}" y="15">{}</text>"#, treatment.read().unwrap().level() * 10, name));
        let (xml, height) = treatment(t, t.read().unwrap().level() as u64 * 400 + 200, 0, 200);
        result.push_str(&xml);
    }

    result.push_str(&format!("</svg>"));

    result
}


fn treatment(treatment: &Arc<RwLock<TreatmentDesigner>>, x: u64, y: u64, width: u64) -> (String, u64) {

    let treatment = treatment.read().unwrap();
    let descriptor = treatment.descriptor();
    let max_iop = *vec![
        descriptor.inputs().len(),
        descriptor.outputs().len(),
        treatment.parameters().len(),
        ].iter().max().unwrap();

    let height = (max_iop as u64 + 2) * 20 + 35;

    let mut result = String::new();

    result.push_str(&format!(r#"<g id="{}" transform="translate({} {})">"#, treatment.name(), x, y));

    result.push_str(&format!(r#"<rect class="treatment" width="{}" height="{}" rx="10" />"#, width, height));
    result.push_str(&format!(r#"<text class="treatment-name" text-anchor="middle" x="{}" y="20">{}</text><text class="treatment-type" text-anchor="middle" x="{}" y="35">{}</text>"#, width / 2, treatment.name(), width / 2, descriptor.identifier().name()));

    let mut y = 45;
    for name in descriptor.inputs().keys().sorted() {
        result.push_str(&format!(r#"<circle class="input" cx="0" cy="{}" r="5"/>"#, y));
        result.push_str(&format!(r#"<text class="input-name" text-anchor="start" x="10" y="{}">{}</text>"#, y+5, name));
        y += 20;
    }

    result.push_str("</g>");

    (result, height)
}
