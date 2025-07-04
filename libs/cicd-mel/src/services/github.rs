use javascript_mel::*;
use melodium_core::{executive::*, *};
use melodium_macro::{mel_data, mel_function, mel_treatment};
use process_mel::command::*;
use regex::{Captures, Regex, Replacer};
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};
use std_mel::data::string_map::*;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum State {
    Pending,
    Success,
    Failure,
    Error,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Pending => write!(f, "pending"),
            State::Success => write!(f, "success"),
            State::Failure => write!(f, "failure"),
            State::Error => write!(f, "error"),
        }
    }
}

#[mel_data(
    traits (PartialEquality Serialize Deserialize Display ToString)
)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StepState(pub State);

impl ToString for StepState {
    fn to_string(&self) -> String {
        format!("{}", self.0)
    }
}

impl Display for StepState {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

#[mel_function]
pub fn pending() -> StepState {
    StepState(State::Pending)
}
#[mel_function]
pub fn success() -> StepState {
    StepState(State::Success)
}

#[mel_function]
pub fn failure() -> StepState {
    StepState(State::Failure)
}

#[mel_function]
pub fn error() -> StepState {
    StepState(State::Error)
}

#[mel_treatment(
    model engine JavaScriptEngine
    input map Block<StringMap>
    output evaluated Block<StringMap>
)]
pub async fn github_map_eval() {
    let engine = JavaScriptEngineModel::into(engine);

    if let Ok(map) = map.recv_one().await.map(|val| {
        GetData::<Arc<dyn Data>>::try_data(val)
            .unwrap()
            .downcast_arc::<StringMap>()
            .unwrap()
    }) {
        let mut updated_map = HashMap::new();
        let var_replacer = VarReplacer { js_engine: &engine };
        for (name, value) in &map.map {
            let regex = github_regex();
            updated_map.insert(
                name.clone(),
                regex.replace_all(value, &var_replacer).to_string(),
            );
        }
        let _ = evaluated
            .send_one(Value::Data(
                Arc::new(StringMap { map: updated_map }) as Arc<dyn Data>
            ))
            .await;
    }
}

#[mel_treatment(
    model engine JavaScriptEngine
    input value Block<string>
    output evaluated Block<string>
)]
pub async fn github_string_eval() {
    let engine = JavaScriptEngineModel::into(engine);

    if let Ok(value) = value
        .recv_one()
        .await
        .map(|val| GetData::<String>::try_data(val).unwrap())
    {
        let var_replacer = VarReplacer { js_engine: &engine };
        let regex = github_regex();
        let _ = evaluated.send_one(regex.replace_all(&value, &var_replacer).to_string().into());
    }
}

#[mel_treatment(
    input shell Block<Option<string>>
    input run Block<string>
    output command Stream<Command>
)]
pub async fn github_command() {
    if let (Ok(shell), Ok(run)) = (
        shell
            .recv_one()
            .await
            .map(|val| GetData::<Option<String>>::try_data(val).unwrap()),
        run.recv_one()
            .await
            .map(|val| GetData::<String>::try_data(val).unwrap()),
    ) {
        let mut file_path = std::env::temp_dir();

        let mut hasher = std::hash::DefaultHasher::new();
        run.hash(&mut hasher);
        let run_hash = format!("{:x}", core::hash::Hasher::finish(&hasher));

        let github_command = match shell.as_ref().map(|s| s.as_str()) {
            Some("bash") => {
                let file_name = format!("{run_hash}.sh");
                file_path.push(&file_name);

                Some(Command {
                    command: "bash".into(),
                    arguments: vec![
                        "--noprofile".into(),
                        "--norc".into(),
                        "-eo".into(),
                        "pipefail".into(),
                        file_path.to_string_lossy().to_string(),
                    ],
                })
            }
            Some("pwsh") => {
                let file_name = format!("{run_hash}.ps1");
                file_path.push(&file_name);

                Some(Command {
                    command: "pwsh".into(),
                    arguments: vec![
                        "-command".into(),
                        format!(
                            ". '{file_path}'",
                            file_path = file_path.to_string_lossy().to_string()
                        ),
                    ],
                })
            }
            Some("python") => {
                let file_name = format!("{run_hash}.py");
                file_path.push(&file_name);

                Some(Command {
                    command: "python".into(),
                    arguments: vec![file_path.to_string_lossy().to_string()],
                })
            }
            Some("sh") => {
                let file_name = format!("{run_hash}.sh");
                file_path.push(&file_name);

                Some(Command {
                    command: "sh".into(),
                    arguments: vec!["-e".into(), file_path.to_string_lossy().to_string()],
                })
            }
            Some("cmd") => {
                let file_name = format!("{run_hash}.cmd");
                file_path.push(&file_name);

                Some(Command {
                    command: std::env::var("ComSpec").unwrap_or_else(|_| "cmd.exe".into()),
                    arguments: vec![
                        "/D".into(),
                        "/E:ON".into(),
                        "/V:OFF".into(),
                        "/S".into(),
                        "/C".into(),
                        format!(
                            r#"CALL "{file_path}.cmd""#,
                            file_path = file_path.to_string_lossy().to_string()
                        ),
                    ],
                })
            }
            Some("powershell") => {
                let file_name = format!("{run_hash}.ps1");
                file_path.push(&file_name);

                Some(Command {
                    command: "powershell".into(),
                    arguments: vec![
                        "-command".into(),
                        format!(
                            ". '{file_path}'",
                            file_path = file_path.to_string_lossy().to_string()
                        ),
                    ],
                })
            }
            #[cfg(target_family = "unix")]
            None => {
                let file_name = format!("{run_hash}.sh");
                file_path.push(&file_name);

                Some(Command {
                    command: "bash".into(),
                    arguments: vec!["-e".into(), file_path.to_string_lossy().to_string()],
                })
            }
            #[cfg(target_family = "windows")]
            None => {
                let file_name = format!("{run_hash}.ps1");
                file_path.push(&file_path);

                Some(Command {
                    command: "pwsh".into(),
                    arguments: vec![
                        "-command".into(),
                        format!(
                            ". '{file_path}'",
                            file_path = file_path.to_string_lossy().to_string()
                        ),
                    ],
                })
            }
            _ => None,
        };

        if let Some(github_command) = github_command {
            if let Ok(_) = std::fs::write(&file_path, run.as_bytes()) {
                let _ = command
                    .send_one(Value::Data(Arc::new(github_command)))
                    .await;
            }
        }
    }
}

static VAR_REGEX: OnceLock<Regex> = OnceLock::new();

fn github_regex() -> &'static Regex {
    VAR_REGEX.get_or_init(|| Regex::new(r#"\$\{\{(.*)\}\}"#).unwrap())
}

struct VarReplacer<'a> {
    pub js_engine: &'a JavaScriptEngineModel,
}

impl<'a> Replacer for &VarReplacer<'a> {
    fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
        let eval = async_std::task::block_on(
            self.js_engine
                .inner()
                .process(serde_json::Value::Null, caps[1].to_string()),
        )
        .map(|res| res.map(|val| val.as_str().map(|s| s.to_string())).ok())
        .ok()
        .flatten()
        .flatten()
        .unwrap_or_default();
        dst.push_str(eval.as_str());
    }
}
