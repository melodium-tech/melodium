use melodium_core::*;
use melodium_macro::{mel_data, mel_function, mel_treatment};
use regex::Regex;
use std::sync::OnceLock;
use std_mel::data::string_map::*;

static VAR_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn environment_variable_regex() -> &'static Regex {
    VAR_REGEX.get_or_init(|| Regex::new(r#"\${([a-zA-Z_][0-9a-zA-Z_]*)}"#).unwrap())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[mel_data(traits(Serialize Deserialize PartialEquality Equality))]
pub struct Environment {
    pub working_directory: Option<string>,
    pub clear_env: bool,
    pub variables: StringMap,
    pub expand_variables: bool,
}

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
    if let Ok(variables) = variables.recv_one().await.map(|val| {
        GetData::<std::sync::Arc<dyn Data>>::try_data(val)
            .unwrap()
            .downcast_arc::<StringMap>()
            .unwrap()
    }) {
        let _ = environment
            .send_one(
                (std::sync::Arc::new(Environment {
                    working_directory,
                    variables: (*variables).clone(),
                    expand_variables,
                    clear_env,
                }) as std::sync::Arc<dyn Data>)
                    .into(),
            )
            .await;
    }
}
