
use crate::logic::descriptor::*;

#[derive(Debug)]
pub struct Output {
    pub svg: String,
    pub x: u64,
    pub y: u64,
}

impl Output {

    pub fn new(output: &OutputDescriptor, treatment_name: &str, x: u64, y: u64) -> Self {

        let mut svg = String::new();

        svg.push_str(&format!(r#"<g id="{}:output:{}" class="output" transform="translate({} {})">"#, treatment_name, output.name(), x, y));

        svg.push_str(&format!(r#"<circle class="output-sym" cx="0" cy="0" r="5" />"#));
        svg.push_str(&format!(r#"<text class="output-name" text-anchor="end" x="-10" y="2.5">{}</text>"#, output.name()));
        
        svg.push_str("</g>");

        Self { svg, x, y }
    }

    pub fn new_self(output: &OutputDescriptor, x: u64, y: u64) -> Self {

        let mut svg = String::new();

        svg.push_str(&format!(r#"<g id="Self:output:{}" class="output" transform="translate({} {})">"#, output.name(), x, y));

        svg.push_str(&format!(r#"<circle class="io self-output" cx="0" cy="0" r="5"/>"#));
        svg.push_str(&format!(r#"<text class="io-name self-output-name" text-anchor="start" x="10" y="2.5">{}</text>"#, output.name()));
        
        svg.push_str("</g>");

        Self { svg, x, y }
    }

}