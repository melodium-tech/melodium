
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

        svg.push_str(&format!(r#"<path class="connection" d="M {} {} L {} {}" />"#, start_x, start_y, end_x, end_y));

        Self { svg, start_x, start_y, end_x, end_y }
    }
}

