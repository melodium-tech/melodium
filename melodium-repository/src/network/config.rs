#[derive(Clone, Debug)]
pub struct Config {
    pub user_agent: String,
    pub base_url: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            user_agent: format!(
                "melodium-repository/{} ({}; {}; {})",
                env!("CARGO_PKG_VERSION"),
                env!("TARGET"),
                env!("TARGET_FEATURE"),
                env!("HOST")
            ),
            base_url: format!(
                "https://repo.melodium.tech/mel/{}",
                env!("CARGO_PKG_VERSION")
            ),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
