
#[derive(Debug)]
pub enum ResultStatus {
    Ok
}

impl Default for ResultStatus {
    fn default() -> Self {
        Self::Ok
    }
}
