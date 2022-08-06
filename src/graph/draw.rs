
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use itertools::Itertools;
use html_escape::encode_text;

use crate::logic::designer::*;
use crate::logic::descriptor::*;

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

    let width = (max_level + 1) * 400;
    result.push_str(&format!(r#"<svg xmlns="http://www.w3.org/2000/svg" height="1200" width="{}"><style>{}</style>"#, width, include_str!("style.css")));

    let mut y = 125;
    for name in sequence.descriptor().inputs().keys().sorted() {
        result.push_str(&format!(r#"<circle class="io self-input" cx="80" cy="{}" r="5"/>"#, y));
        result.push_str(&format!(r#"<text class="io-name self-input-name" text-anchor="end" x="70" y="{}">{}</text>"#, y+5, name));
        y += 20;
    }

    let mut y = 125;
    for name in sequence.descriptor().outputs().keys().sorted() {
        result.push_str(&format!(r#"<circle class="io self-output" cx="{}" cy="{}" r="5"/>"#, width-80, y));
        result.push_str(&format!(r#"<text class="io-name self-output-name" text-anchor="start" x="{}" y="{}">{}</text>"#, width-70, y+5, name));
        y += 20;
    }

    let mut sizes: HashMap<String, (u64, u64)> = HashMap::new();
    let mut levels: HashMap<u64, u64> = HashMap::new();
    for (name, t) in sequence.treatments() {

        let level = t.read().unwrap().level() as u64;
        let y = levels.entry(level).or_default();
        let (xml, height) = treatment(t, (level * 400) + 50, *y, 200);

        sizes.insert(name.clone(), (200, height));
        levels.entry(level).and_modify(|h| *h += height + 70).or_insert(height);

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

    result.push_str(&format!(r#"<g id="{}" class="treatment" transform="translate({} {})">"#, treatment.name(), x, y));

    result.push_str(&format!(r#"<rect class="treatment-bg" width="{}" height="{}" rx="10" />"#, width, height));
    result.push_str(&format!(r#"<text class="treatment-name" text-anchor="middle" x="{}" y="20">{}</text><text class="treatment-type" text-anchor="middle" x="{}" y="35">{}</text>"#, width / 2, treatment.name(), width / 2, descriptor.identifier().name()));

    let mut y = 55;
    for name in descriptor.inputs().keys().sorted() {
        let desc_input = descriptor.inputs().get(name).unwrap();
        /*result.push_str(&format!(r#"<circle class="io input" cx="0" cy="{}" r="5"/>"#, y));
        result.push_str(&format!(r#"<text class="io-name input-name" text-anchor="start" x="10" y="{}">{}</text>"#, y+5, name));*/
        result.push_str(&input(desc_input, 0, y));
        y += 20;
    }

    let mut y = 55;
    for name in descriptor.outputs().keys().sorted() {
        let desc_output = descriptor.outputs().get(name).unwrap();
        /*result.push_str(&format!(r#"<circle class="io output" cx="{}" cy="{}" r="5"/>"#, width, y));
        result.push_str(&format!(r#"<text class="io-name output-name" text-anchor="end" x="{}" y="{}">{}</text>"#, width-10, y+5, name));*/
        result.push_str(&output(desc_output, width, y));
        y += 20;
    }

    let mut y = 55;
    for name in treatment.parameters().keys().sorted() {
        let param = treatment.parameters().get(name).unwrap().read().unwrap();

        result.push_str(&parameter(&param, width/2, y+5));

        //result.push_str(&format!(r#"<text class="param" text-anchor="middle" x="{}" y="{}"><tspan class="param-name">{}</tspan> = <tspan class="param-value">&quot;truc&quot;</tspan></text>"#, width/2, y+5, name));
        y += 20;
    }

    result.push_str("</g>");

    (result, height)
}

fn input(input: &InputDescriptor, x: u64, y: u64) -> String {

    let mut result = String::new();

    result.push_str(&format!(r#"<g class="input" transform="translate({} {})">"#, x, y));

    result.push_str(&format!(r#"<circle class="input-sym" cx="0" cy="0" r="5" />"#));
    result.push_str(&format!(r#"<text class="input-name" text-anchor="start" x="10" y="5">{}</text>"#, input.name()));
    
    result.push_str("</g>");

    result
}

fn output(output: &OutputDescriptor, x: u64, y: u64) -> String {

    let mut result = String::new();

    result.push_str(&format!(r#"<g class="output" transform="translate({} {})">"#, x, y));

    result.push_str(&format!(r#"<circle class="output-sym" cx="0" cy="0" r="5" />"#));
    result.push_str(&format!(r#"<text class="output-name" text-anchor="end" x="-10" y="5">{}</text>"#, output.name()));
    
    result.push_str("</g>");

    result
}

fn parameter(param: &ParameterDesigner, x: u64, y: u64) -> String {

    let mut result = String::new();

    result.push_str(&format!(r#"<g class="param" transform="translate({} {})">"#, x, y));

    result.push_str(&format!(r#"<text class="param-text" text-anchor="middle" x="0" y="5"><tspan class="param-name">{}</tspan> = <tspan class="param-value">{}</tspan></text>"#, param.name(), value(&param.value().as_ref().unwrap(), 12).0));
    
    result.push_str("</g>");

    result
}

fn value(val: &ValueDesigner, max_chars: usize) -> (String, String) {

    let mut svg = String::new();
    let mut raw = String::new();

    match val {
        ValueDesigner::Raw(v) => {
            let all = v.to_string();
            raw.push_str(&all);

            let part = 
                if all.chars().count() <= max_chars { all }
                else { format!("{}…", all.chars().take(max_chars-1).collect::<String>()) };
            svg.push_str(&format!(r#"<tspan class="value value-value">{}</tspan>"#, encode_text(&part)));
            
        }
        ValueDesigner::Variable(v) => {
            let all = v.to_string();
            raw.push_str(&all);

            let part = 
                if all.chars().count() <= max_chars { all }
                else { format!("{}…", all.chars().take(max_chars-1).collect::<String>()) };
            svg.push_str(&format!(r#"<tspan class="value value-variable">{}</tspan>"#, encode_text(&part)));
        }
        ValueDesigner::Context((n, v)) => {
            let all = format!("{}[{}]", n, v);
            raw.push_str(&all);

            let part = 
                if all.chars().count() <= max_chars { all }
                else { format!("{}…", all.chars().take(max_chars-1).collect::<String>()) };
            svg.push_str(&format!(r#"<tspan class="value value-context">{}</tspan>"#, encode_text(&part)));
        }
        ValueDesigner::Function(f, v) => {

            let all_values = v.iter().map(|v| value(v, usize::MAX).1).collect::<Vec<_>>().join(", ");
            let all = format!("{}({})", f.identifier().name(), all_values);
            raw.push_str(&all);

            let part = 
                if all.chars().count() <= max_chars { all }
                else { format!("{}…", all.chars().take(max_chars-1).collect::<String>()) };
            svg.push_str(&format!(r#"<tspan class="value value-function">{}</tspan>"#, encode_text(&part)));
        }
    }

    (svg, raw)
}
