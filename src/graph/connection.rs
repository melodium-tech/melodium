
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

        let diff = start_y.abs_diff(end_y);
        if diff == 0 {
            path.push_str(&format!("M {} {} H {}", start_x, start_y, end_x));
        }
        else if diff < 25 {
            let x_change = end_x - 25;

            path.push_str(&format!("M {} {} H {} C {} {}, {} {}, {} {}",
                start_x, start_y, x_change - 25,
                x_change, start_y,
                x_change, end_y,
                end_x, end_y
            ));
        }
        else {
            let x_change = end_x - 25;

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
        
        svg.push_str(&format!(r#"<path class="connection" d="{}" />"#, path));

        Self { svg, start_x, start_y, end_x, end_y }
    }
}

