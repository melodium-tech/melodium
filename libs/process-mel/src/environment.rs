use melodium_core::*;
use melodium_macro::{mel_data, mel_function, mel_treatment};
use regex::Regex;
use std::sync::OnceLock;
use std_mel::data::string_map::*;

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
#[mel_data(traits(Serialize Deserialize PartialEquality Equality))]
pub struct Environment {
    pub working_directory: Option<string>,
    pub clear_env: bool,
    pub variables: StringMap,
    pub expand_variables: bool,
}

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
    }
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
            }) as std::sync::Arc<dyn Data>)
            .await;
    }
}
