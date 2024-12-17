use melodium_core::*;
use melodium_macro::{mel_data, mel_function};
use std_mel::data::*;

#[derive(Debug, PartialEq, Serialize)]
#[mel_data]
pub struct Environment {
    pub working_directory: Option<string>,
    pub clear_env: bool,
    pub variables: Map,
}

#[mel_function]
pub fn environment(
    variables: Map,
    working_directory: Option<string>,
    clear_env: bool,
) -> Environment {
    Environment {
        working_directory,
        variables,
        clear_env,
    }
}
