
use crate::logic::descriptor::*;

#[derive(Debug)]
pub struct Input {
    pub svg: String,
    pub x: u64,
    pub y: u64,
}

impl Input {

    pub fn new(input: &InputDescriptor, treatment_name: &str, x: u64, y: u64) -> Self {

        let mut svg = String::new();
    
        svg.push_str(&format!(r#"<g id="{}:input:{}" class="input" transform="translate({} {})">"#, treatment_name, input.name(), x, y));
    
        svg.push_str(&format!(r#"<circle class="input-sym" cx="0" cy="0" r="5" />"#));
        svg.push_str(&format!(r#"<text class="input-name" text-anchor="start" x="10" y="2.5">{}</text>"#, input.name()));
        
        svg.push_str("</g>");
    
        Self { svg, x, y }
    }

    pub fn new_self(input: &InputDescriptor, x: u64, y: u64) -> Self {

        let mut svg = String::new();
    
        svg.push_str(&format!(r#"<g id="Self:input:{}" class="input" transform="translate({} {})">"#, input.name(), x, y));
    
        svg.push_str(&format!(r#"<circle class="io self-input" cx="0" cy="0" r="5"/>"#));
        svg.push_str(&format!(r#"<text class="io-name self-input-name" text-anchor="end" x="-10" y="2.5">{}</text>"#, input.name()));
        
        svg.push_str("</g>");
    
        Self { svg, x, y }
    }

}
