use melodium_core::common::{
    descriptor::DataType,
    executive::{Secret as ExecutiveSecret, SecretAccess},
};
use melodium_core::*;
use melodium_macro::{mel_data, mel_function, mel_treatment};
use regex::Regex;
use std::collections::BTreeMap;
use std::sync::OnceLock;
use std_mel::data::map::*;
use std_mel::data::string_map::*;
use zeroize::Zeroizing;

static VAR_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn environment_variable_regex() -> &'static Regex {
    VAR_REGEX.get_or_init(|| Regex::new(r#"\$\{([a-zA-Z_][0-9a-zA-Z_]*)\}"#).unwrap())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Execution environment for a subprocess.
///
/// - `working_directory`: optional directory to set as the process working directory.
/// - `clear_env`: when `true`, the subprocess inherits no environment variables from the parent process.
/// - `variables`: the environment variables to set.
/// - `expand_variables`: when `true`, `${VAR}` references in variable values are expanded using the parent environment.
/// - `secret_variables`: the environment variables whose values are secrets, see `|with_secret_variables`.
/// - `secret_stdin`: a secret written to the standard input of commands, see `|with_secret_stdin`.
///
/// Secrets are revealed by the treatment running each command, and never put in command arguments.
#[mel_data(traits(Serialize Deserialize PartialEquality Equality))]
pub struct Environment {
    pub working_directory: Option<string>,
    pub clear_env: bool,
    pub variables: StringMap,
    pub expand_variables: bool,
    /// None for an entry that is not a `Secret<string>`.
    #[serde(default)]
    pub secret_variables: BTreeMap<String, Option<ExecutiveSecret>>,
    #[serde(default)]
    pub secret_stdin: Option<ExecutiveSecret>,
}

/// Values of the secrets of an environment, revealed to run one command,
/// and overwritten when dropped.
#[derive(Default)]
pub struct RevealedSecrets {
    pub variables: Vec<(String, Zeroizing<String>)>,
    pub stdin: Option<Zeroizing<String>>,
}

impl RevealedSecrets {
    /// Reveals the secrets of `environment` on behalf of `access`.
    pub async fn reveal(
        environment: Option<&Environment>,
        access: &SecretAccess,
    ) -> Result<Self, String> {
        let mut revealed = Self::default();
        let Some(environment) = environment else {
            return Ok(revealed);
        };
        for (name, secret) in &environment.secret_variables {
            let Some(secret) = secret else {
                return Err(format!("secret variable '{name}' is not a Secret<string>"));
            };
            let value = secret
                .reveal_str(access, |value| Zeroizing::new(value.to_string()))
                .await
                .map_err(|error| format!("secret variable '{name}': {error}"))?;
            revealed.variables.push((name.clone(), value));
        }
        if let Some(secret) = &environment.secret_stdin {
            revealed.stdin = Some(
                secret
                    .reveal_str(access, |value| Zeroizing::new(value.to_string()))
                    .await
                    .map_err(|error| format!("secret input: {error}"))?,
            );
        }
        Ok(revealed)
    }

    /// Tells if there is anything to write to the standard input of commands.
    pub fn has_input(&self) -> bool {
        !self.variables.is_empty() || self.stdin.is_some()
    }

    /// Gives `command` (name and arguments) run through a POSIX shell that sets the secret
    /// variables from its standard input, for executors that cannot set environment variables
    /// otherwise than through arguments. Only variable names appear in the arguments.
    ///
    /// `shell_input` gives what to write first to the standard input.
    pub fn shell_command(&self, command: Vec<String>) -> Vec<String> {
        if self.variables.is_empty() {
            return command;
        }
        let mut wrapped = vec![
            "/bin/sh".to_string(),
            "-c".to_string(),
            SECRET_VARIABLES_SCRIPT.to_string(),
            "melodium-secrets".to_string(),
        ];
        wrapped.extend(self.variables.iter().map(|(name, _)| name.clone()));
        wrapped.push("--".to_string());
        wrapped.extend(command);
        wrapped
    }

    /// Gives what to write first to the standard input of a command made by `shell_command`:
    /// one line per secret variable, each byte written as an octal escape, then the secret input.
    pub fn shell_input(&self) -> Zeroizing<Vec<u8>> {
        let capacity = self
            .variables
            .iter()
            .map(|(_, value)| value.len() * 5 + 1)
            .sum::<usize>()
            + self.stdin.as_ref().map(|stdin| stdin.len()).unwrap_or(0);
        let mut input = Zeroizing::new(Vec::with_capacity(capacity));
        for (_, value) in &self.variables {
            for byte in value.as_bytes() {
                input.extend_from_slice(&[
                    b'\\',
                    b'0',
                    b'0' + (byte >> 6),
                    b'0' + ((byte >> 3) & 7),
                    b'0' + (byte & 7),
                ]);
            }
            input.push(b'\n');
        }
        if let Some(stdin) = &self.stdin {
            input.extend_from_slice(stdin.as_bytes());
        }
        input
    }
}

/// Exports the variables named before `--` from lines of standard input decoded by
/// `printf %b`, then runs the command after `--`, keeping the rest of standard input.
/// `read` takes one byte at a time from pipes, so it never reads past its line.
const SECRET_VARIABLES_SCRIPT: &str = r#"while [ "$1" != -- ]; do IFS= read -r line || exit 125; value=$(printf '%b' "$line"; printf x); export "$1=${value%x}"; shift; done; shift; exec "$@""#;

/// Build an `Environment` value from explicit parameters.
///
/// - `variables`: key-value pairs to expose to the subprocess.
/// - `working_directory`: optional working directory.
/// - `expand_variables`: expand `${VAR}` references in variable values.
/// - `clear_env`: inherit no variables from the parent process.
#[mel_function]
pub fn environment(
    variables: StringMap,
    working_directory: Option<string>,
    expand_variables: bool,
    clear_env: bool,
) -> Environment {
    Environment {
        working_directory,
        variables,
        expand_variables,
        clear_env,
        secret_variables: BTreeMap::new(),
        secret_stdin: None,
    }
}

/// Add secret variables to `environment`.
///
/// Each entry of `secret_variables` gives the `Secret<string>` value of a variable, revealed
/// only when a command is run, and never put in command arguments. Running a command fails if
/// an entry is not a `Secret<string>`. Variables given here replace those of the same name.
#[mel_function]
pub fn with_secret_variables(mut environment: Environment, secret_variables: Map) -> Environment {
    for (name, value) in secret_variables.map {
        let secret = match value {
            Value::Secret(secret) if secret.datatype() == &DataType::String => Some(secret),
            _ => None,
        };
        environment.secret_variables.insert(name, secret);
    }
    environment
}

/// Write `secret` to the standard input of commands run with `environment`, before anything else.
///
/// It is revealed only when a command is run.
#[mel_function]
pub fn with_secret_stdin(mut environment: Environment, secret: Secret<string>) -> Environment {
    environment.secret_stdin = Some(secret);
    environment
}

/// Convert a `StringMap` block into an `Environment` block.
///
/// `variables` supplies the key-value pairs; other environment properties come from the
/// constant parameters `clear_env`, `expand_variables`, and `working_directory`.
///
/// ```mermaid
/// graph LR
///     T("mapEnvironment()")
///     V["〈🟦〉"] -->|variables| T
///     T -->|environment| E["〈🟨〉"]
///     style V fill:#ffffff,stroke:#ffffff
///     style E fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input variables Block<StringMap>
    output environment Block<Environment>
    default clear_env false
    default expand_variables false
)]
pub async fn map_environment(
    clear_env: bool,
    expand_variables: bool,
    working_directory: Option<string>,
) {
    if let Ok(variables) = variables.recv_one_as::<std::sync::Arc<StringMap>>().await {
        let _ = environment
            .send_one_as(std::sync::Arc::new(Environment {
                working_directory,
                variables: (*variables).clone(),
                expand_variables,
                clear_env,
                secret_variables: BTreeMap::new(),
                secret_stdin: None,
            }) as std::sync::Arc<dyn Data>)
            .await;
    }
}

/// Convert a `StringMap` block and a working directory block into an `Environment` block.
///
/// Like `map_environment`, but also accepts `working_directory` as a streamed `Block<Option<string>>`
/// instead of a constant parameter.
///
/// ```mermaid
/// graph LR
///     T("mapFullEnvironment()")
///     V["〈🟦〉"] -->|variables| T
///     W["〈🟨〉"] -->|working_directory| T
///     T -->|environment| E["〈🟩〉"]
///     style V fill:#ffffff,stroke:#ffffff
///     style W fill:#ffffff,stroke:#ffffff
///     style E fill:#ffffff,stroke:#ffffff
/// ```
#[mel_treatment(
    input variables Block<StringMap>
    input working_directory Block<Option<string>>
    output environment Block<Environment>
    default clear_env false
    default expand_variables false
)]
pub async fn map_full_environment(clear_env: bool, expand_variables: bool) {
    if let (Ok(working_directory), Ok(variables)) = (
        working_directory.recv_one_as::<Option<String>>().await,
        variables.recv_one_as::<std::sync::Arc<StringMap>>().await,
    ) {
        let _ = environment
            .send_one_as(std::sync::Arc::new(Environment {
                working_directory,
                variables: (*variables).clone(),
                expand_variables,
                clear_env,
                secret_variables: BTreeMap::new(),
                secret_stdin: None,
            }) as std::sync::Arc<dyn Data>)
            .await;
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::io::Write;
    use std::process::{Command, Stdio};

    fn secrets() -> RevealedSecrets {
        RevealedSecrets {
            variables: vec![
                (
                    "TOKEN".to_string(),
                    Zeroizing::new("t0k3n with spaces, quotes \" ' and \\ backslash".to_string()),
                ),
                (
                    "KEY".to_string(),
                    Zeroizing::new("-----BEGIN KEY-----\nline\n-----END KEY-----\n\n".to_string()),
                ),
            ],
            stdin: Some(Zeroizing::new("input\n".to_string())),
        }
    }

    /// Runs the wrapped command with `shell` as `/bin/sh`, giving its standard output.
    fn run(shell: &str, secrets: &RevealedSecrets, rest: &[u8]) -> String {
        let mut command = secrets.shell_command(vec![
            "sh".to_string(),
            "-c".to_string(),
            r#"printf '%s|%s|' "$TOKEN" "$KEY"; cat"#.to_string(),
        ]);
        command[0] = shell.to_string();
        let mut child = Command::new(&command[0])
            .args(&command[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(&secrets.shell_input()).unwrap();
        stdin.write_all(rest).unwrap();
        drop(stdin);
        String::from_utf8(child.wait_with_output().unwrap().stdout).unwrap()
    }

    #[test]
    fn secret_variables_go_through_standard_input() {
        let secrets = secrets();
        let command = secrets.shell_command(vec!["cmd".to_string()]);
        assert!(!command.iter().any(|arg| arg.contains("t0k3n")));
        assert_eq!(&command[4..], ["TOKEN", "KEY", "--", "cmd"]);

        for shell in ["/bin/sh", "/usr/bin/dash", "/usr/bin/busybox"] {
            if !std::path::Path::new(shell).exists() {
                continue;
            }
            let shell = if shell.ends_with("busybox") {
                // `busybox sh` is reached through a link named `sh`.
                let link =
                    std::env::temp_dir().join(format!("melodium_busybox_{}", std::process::id()));
                std::fs::create_dir_all(&link).unwrap();
                let sh = link.join("sh");
                let _ = std::os::unix::fs::symlink(shell, &sh);
                sh.to_string_lossy().into_owned()
            } else {
                shell.to_string()
            };
            assert_eq!(
                run(&shell, &secrets, b"data"),
                "t0k3n with spaces, quotes \" ' and \\ backslash|-----BEGIN KEY-----\nline\n-----END KEY-----\n\n|input\ndata",
                "with {}",
                shell
            );
        }
    }

    #[test]
    fn commands_without_secret_variables_are_unchanged() {
        let secrets = RevealedSecrets {
            variables: Vec::new(),
            stdin: Some(Zeroizing::new("input".to_string())),
        };
        assert_eq!(secrets.shell_command(vec!["cmd".to_string()]), ["cmd"]);
        assert_eq!(&*secrets.shell_input(), b"input");
        assert!(secrets.has_input());
        assert!(!RevealedSecrets::default().has_input());
    }
}
