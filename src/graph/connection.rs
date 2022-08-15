
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use itertools::Itertools;

use crate::logic::designer::*;
use crate::logic::descriptor::*;

pub struct Connection {

    pub svg: String,

    pub start_x: u64,
    pub start_y: u64,
    pub end_x: u64,
    pub end_y: u64,
}

impl Connection {

    pub fn new(start_x: u64, start_y: u64, end_x: u64, end_y: u64) -> Self {

        let mut svg = String::new();
        let mut path = String::new();

        //svg.push_str(&format!(r#"<path class="connection" d="M {} {} L {} {}" />"#, start_x, start_y, end_x, end_y));

        if start_y != end_y {

            let x_change = end_x - 25;
            //let y_change = end_y - 25;

            let direction: i64 = if start_y > end_y { -25 } else { 25 };

            path.push_str(&format!("M {} {} H {} Q {} {}, {} {} V {} Q {} {}, {} {}",
                start_x, start_y, x_change - 25,
                x_change, start_y,
                x_change, start_y as i64 + direction,
                end_y as i64 - direction,
                x_change, end_y,
                end_x, end_y
            ));
        }
        else {
            path.push_str(&format!("M {} {} H {}", start_x, start_y, end_x));
        }
        
        svg.push_str(&format!(r#"<path class="connection" d="{}" />"#, path));

        Self { svg, start_x, start_y, end_x, end_y }
    }
}

