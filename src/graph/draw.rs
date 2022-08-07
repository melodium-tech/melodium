
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use itertools::Itertools;

use crate::logic::designer::*;
use crate::logic::descriptor::*;
use crate::graph::treatment::Treatment;
use crate::graph::input::Input;
use crate::graph::output::Output;

pub fn draw(sequence: Arc<RwLock<SequenceDesigner>>) -> String {

    let sequence = sequence.read().unwrap();

    let mut min_level = usize::MAX;
    let mut max_level = usize::MIN;
    for (_, t) in sequence.treatments() {
        
        let level = t.read().unwrap().level();

        if max_level < level {
            max_level = level;
        }

        if min_level > level {
            min_level = level;
        }
    }

    let width = (max_level as u64 + 1) * 400;

    let mut self_inputs = HashMap::new();
    let mut y = 125;
    for name in sequence.descriptor().inputs().keys().sorted() {

        let input = Input::new_self(sequence.descriptor().inputs().get(name).unwrap(), 80, y);

        self_inputs.insert(name.clone(), input);
        y += 20;
    }

    let mut self_outputs = HashMap::new();
    let mut y = 125;
    for name in sequence.descriptor().outputs().keys().sorted() {

        let output = Output::new_self(sequence.descriptor().outputs().get(name).unwrap(), width-80, y);

        self_outputs.insert(name.clone(), output);
        y += 20;
    }

    let mut levels: HashMap<u64, u64> = HashMap::new();
    let mut treatments = HashMap::new();
    for name in sequence.treatments().keys().sorted() {

        let t = sequence.treatments().get(name).unwrap();

        let level = t.read().unwrap().level() as u64;
        let y = levels.entry(level).or_default();
        let treatment = Treatment::new(t, (level * 400) + 50, *y, 200);

        levels.entry(level).and_modify(|h| *h += treatment.height + 70).or_insert(treatment.height);

        treatments.insert(name.clone(), treatment);
    }

    let height = *levels.iter().map(|(_, h)| h).max().unwrap_or(&70);


    let mut svg = String::new();
    svg.push_str(&format!(r#"<svg xmlns="http://www.w3.org/2000/svg" height="{}" width="{}"><style>{}</style>"#, height, width, include_str!("style.css")));

    for (_, i) in self_inputs {
        svg.push_str(&i.svg);
    }

    for (_, o) in self_outputs {
        svg.push_str(&o.svg);
    }

    for (_, t) in treatments {
        svg.push_str(&t.svg);
    }

    svg.push_str(&format!("</svg>"));

    svg
}
