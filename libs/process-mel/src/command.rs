use melodium_core::*;
use melodium_macro::{mel_data, mel_function};

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
#[mel_data]
pub struct Command {
    pub command: string,
    pub arguments: Vec<string>,
}

#[mel_function]
pub fn command(command: string, arguments: Vec<string>) -> Command {
    Command { command, arguments }
}
