use melodium_core::*;
use melodium_macro::{mel_data, mel_function};

/// A command with its executable name and argument list.
#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord, Serialize, Deserialize)]
#[mel_data(traits(Serialize Deserialize PartialEquality PartialOrder Equality Order))]
pub struct Command {
    pub command: String,
    pub arguments: Vec<String>,
}

/// Build a `Command` from an explicit executable name and argument list.
#[mel_function]
pub fn command(command: string, arguments: Vec<string>) -> Command {
    Command { command, arguments }
}

/// Parse a single shell-quoted command line into a `Command`.
///
/// Returns `None` if `line` cannot be parsed as a valid shell command (e.g. unmatched quotes).
#[mel_function]
pub fn raw_command(line: string) -> Option<Command> {
    let splitted = shlex::split(&line)?;
    let command = splitted.first().cloned()?;
    let arguments = splitted.into_iter().skip(1).collect();
    Some(Command { command, arguments })
}

/// Parse multiple shell-quoted command lines into a `Vec<Command>`.
///
/// Returns an empty `Vec` if any line cannot be parsed or yields no tokens.
/// Use `|checked_raw_commands` if you need to detect parse failures explicitly.
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

/// Parse multiple shell-quoted command lines, returning `None` on any parse failure.
///
/// Returns `None` if any line cannot be parsed or contains no tokens.
/// Unlike `|raw_commands`, this function propagates errors rather than silently returning an empty `Vec`.
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
