
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

    result.push_str(&format!(r#"<svg xmlns="http://www.w3.org/2000/svg" height="1200" width="{}"><style>{}</style>"#, (max_level + 1) * 100, include_str!("style.css")));

    for (_name, t) in sequence.treatments() {
        //result.push_str(&format!(r#"<text x="{}" y="15">{}</text>"#, treatment.read().unwrap().level() * 10, name));
        result.push_str(&treatment(t, t.read().unwrap().level() as u64 * 100, 0));
    }

    result.push_str(&format!("</svg>"));

    result
}


fn treatment(treatment: &Arc<RwLock<TreatmentDesigner>>, x: u64, y: u64) -> String {

    let treatment = treatment.read().unwrap();
    let descriptor = treatment.descriptor();
    let max_iop = *vec![
        descriptor.inputs().len(),
        descriptor.outputs().len(),
        treatment.parameters().len(),
        ].iter().max().unwrap();

    let width = 100;
    let height = (max_iop as u64 + 2) * 20;

    let mut result = String::new();

    result.push_str(&format!(r#"<g id="{}" transform="translate({} {})">"#, treatment.name(), x, y));

    result.push_str(&format!(r#"<rect class="treatment" width="{}" height="{}" rx="15" />"#, width, height));
    result.push_str(&format!(r#"<text class="treatment-name" text-anchor="middle" x="{}" y="20">{}</text><text class="treatment-type" text-anchor="middle" x="{}" y="35">{}</text>"#, width / 2, treatment.name(), width / 2, descriptor.identifier().name()));

    result.push_str("</g>");

    result
}
