
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use itertools::Itertools;

use crate::logic::designer::*;
use crate::logic::descriptor::*;
use crate::graph::input::Input;
use crate::graph::output::Output;
use crate::graph::value::value;

pub struct Treatment {
    pub svg: String,
    pub x: u64,
    pub y: u64,
    pub width: u64,
    pub height: u64,

    pub inputs: HashMap<String, Input>,
    pub outputs: HashMap<String, Output>,
}

impl Treatment {

    pub fn new(treatment: &Arc<RwLock<TreatmentDesigner>>, x: u64, y: u64, width: u64) -> Self {

        let treatment = treatment.read().unwrap();
        let descriptor = treatment.descriptor();
        let max_iop = *vec![
            descriptor.inputs().len(),
            descriptor.outputs().len(),
            treatment.parameters().len(),
            ].iter().max().unwrap();
    
        let height = (max_iop as u64 + 2) * 20 + 35;
    
        let mut svg = String::new();
    
        svg.push_str(&format!(r#"<g id="{}" class="treatment" transform="translate({} {})">"#, treatment.name(), x, y));
    
        svg.push_str(&format!(r#"<rect class="treatment-bg" width="{}" height="{}" rx="10" />"#, width, height));
        svg.push_str(&format!(r#"<text class="treatment-name" text-anchor="middle" x="{}" y="20">{}</text><text class="treatment-type" text-anchor="middle" x="{}" y="35">{}</text>"#, width / 2, treatment.name(), width / 2, descriptor.identifier().name()));
    
        let mut inputs = HashMap::new();
        let mut i_y = 55;
        for name in descriptor.inputs().keys().sorted() {
            let desc_input = descriptor.inputs().get(name).unwrap();

            let input = Input::new(desc_input, 0, i_y);
            svg.push_str(&input.svg);

            inputs.insert(name.clone(), input);
            i_y += 20;
        }
    
        let mut outputs = HashMap::new();
        let mut o_y = 55;
        for name in descriptor.outputs().keys().sorted() {
            let desc_output = descriptor.outputs().get(name).unwrap();

            let output = Output::new(desc_output, width, o_y);
            svg.push_str(&output.svg);

            outputs.insert(name.clone(), output);
            o_y += 20;
        }
    
        let mut p_y = 55;
        for name in treatment.parameters().keys().sorted() {
            let param = treatment.parameters().get(name).unwrap().read().unwrap();
    
            svg.push_str(&Self::parameter(&param, width/2, p_y+5));
    
            p_y += 20;
        }
    
        svg.push_str("</g>");
    
        Self { svg, x, y, width, height, inputs, outputs }
    }

    fn parameter(param: &ParameterDesigner, x: u64, y: u64) -> String {

        let mut result = String::new();
    
        result.push_str(&format!(r#"<g class="param" transform="translate({} {})">"#, x, y));
    
        result.push_str(&format!(r#"<text class="param-text" text-anchor="middle" x="0" y="5"><tspan class="param-name">{}</tspan> = <tspan class="param-value">{}</tspan></text>"#, param.name(), value(&param.value().as_ref().unwrap(), 12).0));
        
        result.push_str("</g>");
    
        result
    }

}

