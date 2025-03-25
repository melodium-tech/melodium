use melodium_core::*;
use melodium_macro::{mel_data, mel_function};

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
#[mel_data(traits(Serialize Deserialize PartialEquality PartialOrder Equality Order))]
pub struct Command {
    pub command: String,
    pub arguments: Vec<String>,
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
pub fn raw_commands(lines: Vec<string>) -> Vec<Command> {
    let mut commands = vec![];
    for line in lines {
        if let Some(splitted) = shlex::split(&line) {
            if let Some(command) = splitted.first().cloned() {
                let arguments = splitted.into_iter().skip(1).collect();
                commands.push(Command { command, arguments });
            } else {
                return Vec::new();
            }
        } else {
            return Vec::new();
        }
    }
    commands
}

#[mel_function]
pub fn checked_raw_commands(lines: Vec<string>) -> Option<Vec<Command>> {
    let mut commands = vec![];
    for line in lines {
        let splitted = shlex::split(&line)?;
        let command = splitted.first().cloned()?;
        let arguments = splitted.into_iter().skip(1).collect();
        commands.push(Command { command, arguments });
    }
    Some(commands)
}
