use melodium_core::*;
use melodium_macro::{mel_data, mel_function, mel_treatment};
use std_mel::data::string_map::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[mel_data(traits(Serialize Deserialize PartialEquality Equality))]
pub struct Environment {
    pub working_directory: Option<string>,
    pub clear_env: bool,
    pub variables: StringMap,
}

#[mel_function]
pub fn environment(
    variables: StringMap,
    working_directory: Option<string>,
    clear_env: bool,
) -> Environment {
    Environment {
        working_directory,
        variables,
        clear_env,
    }
}

#[mel_treatment(
    input variables Block<StringMap>
    output environment Block<Environment>
    default clear_env false
)]
pub async fn map_environment(clear_env: bool, working_directory: Option<string>) {
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
                    clear_env,
                }) as std::sync::Arc<dyn Data>)
                    .into(),
            )
            .await;
    }
}
