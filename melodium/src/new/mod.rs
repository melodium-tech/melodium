use melodium_common::descriptor::Version;
use std::collections::HashMap;

mod cicd;
mod raw;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
pub enum Template {
    #[default]
    Raw,
    Cicd,
}

pub fn template(
    template: Template,
    program_name: &str,
    version: &Version,
) -> HashMap<String, Vec<u8>> {
    match template {
        Template::Raw => raw::raw_pattern(program_name, version),
        Template::Cicd => cicd::cicd_pattern(program_name, version),
    }
}
