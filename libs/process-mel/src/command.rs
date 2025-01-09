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

#[mel_function]
pub fn raw_command(line: string) -> Option<Command> {
    let splitted = shlex::split(&line)?;
    let command = splitted.first().cloned()?;
    let arguments = splitted.into_iter().skip(1).collect();
    Some(Command { command, arguments })
}

#[mel_function]
pub fn raw_commands(lines: Vec<string>) -> Option<Vec<Command>> {
    let mut commands = vec![];
    for line in lines {
        let splitted = shlex::split(&line)?;
        let command = splitted.first().cloned()?;
        let arguments = splitted.into_iter().skip(1).collect();
        commands.push(Command { command, arguments });
    }
    Some(commands)
}
