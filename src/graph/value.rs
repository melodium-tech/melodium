
use html_escape::encode_text;

use crate::logic::designer::*;

pub fn value(val: &ValueDesigner, max_chars: usize) -> (String, String) {

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

