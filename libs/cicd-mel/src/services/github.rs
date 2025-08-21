use javascript_mel::*;
use json_mel::*;
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
    model contexts JavaScriptEngine
    default assume false
    input map Block<StringMap>
    output evaluated Block<StringMap>
)]
pub async fn github_map_eval(assume: bool, local_context: Json) {
    let engine = JavaScriptEngineModel::into(contexts);

    if let Ok(map) = map.recv_one().await.map(|val| {
        GetData::<Arc<dyn Data>>::try_data(val)
            .unwrap()
            .downcast_arc::<StringMap>()
            .unwrap()
    }) {
        let updated_map = map_eval(&map.map, &local_context.0, &engine, assume);

        let _ = evaluated
            .send_one(Value::Data(
                Arc::new(StringMap { map: updated_map }) as Arc<dyn Data>
            ))
            .await;
    }
}

#[mel_treatment(
    model contexts JavaScriptEngine
    default assume false
    input value Block<string>
    output evaluated Block<string>
)]
pub async fn github_string_eval(assume: bool, local_context: Json) {
    let engine = JavaScriptEngineModel::into(contexts);

    if let Ok(value) = value
        .recv_one()
        .await
        .map(|val| GetData::<String>::try_data(val).unwrap())
    {
        let var_replacer = VarReplacer {
            js_engine: &engine,
            local_context: &local_context.0,
        };

        let regex = github_regex();
        let result = if assume {
            if regex.is_match(&value) {
                regex.replace_all(&value, &var_replacer).to_string()
            } else {
                eval_js(&engine, &local_context.0, &value)
            }
        } else {
            regex.replace_all(&value, &var_replacer).to_string()
        };

        let _ = evaluated.send_one(result.into()).await;
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
                        "-xeo".into(),
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
                    arguments: vec!["-xe".into(), file_path.to_string_lossy().to_string()],
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
                    arguments: vec!["-xe".into(), file_path.to_string_lossy().to_string()],
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
            if let Ok(_) = std::fs::write(&file_path, run.as_bytes())
                .inspect_err(|err| eprintln!("Write error: {err}"))
            {
                eprintln!("File written: {}", file_path.as_os_str().to_string_lossy());
                let _ = command
                    .send_one(Value::Data(Arc::new(github_command)))
                    .await;
            }
        }
    }
}

#[mel_treatment(
    input workflow_id Block<string>
    input step_id Block<string>
    output variables Block<StringMap>
)]
pub async fn github_get_env() {
    if let (Ok(workflow_id), Ok(step_id)) = (
        workflow_id
            .recv_one()
            .await
            .map(|val| GetData::<String>::try_data(val).unwrap()),
        step_id
            .recv_one()
            .await
            .map(|val| GetData::<String>::try_data(val).unwrap()),
    ) {
        let env_file = env_file(&workflow_id);

        let mut map = HashMap::new();
        map.insert("GITHUB_ENV".to_string(), env_file.clone());
        map.insert(
            "GITHUB_OUTPUT".to_string(),
            output_file(&workflow_id, &step_id),
        );
        map.insert("GITHUB_WORKSPACE".to_string(), workspace_dir(&workflow_id));

        if let Ok(env) = dotenvy::from_filename_iter(env_file) {
            for item in env {
                if let Ok((key, val)) = item {
                    map.insert(key, val);
                }
            }
        }

        let _ = variables
            .send_one(Value::Data(Arc::new(StringMap { map })))
            .await;
    }
}

#[mel_treatment(
    model contexts JavaScriptEngine

    input trigger_release Block<void>
    output result Block<Json>
    output success Block<void>
    output failure Block<void>
    output finished Block<void>
)]
pub async fn github_job_result(name: string, outputs: StringMap, local_context: Json) {
    let engine = JavaScriptEngineModel::into(contexts);

    let _ = trigger_release.recv_one().await;

    let outputs = map_eval(&outputs.map, &local_context.0, &engine, false);

    if let Ok(Ok(job_result)) = engine
        .inner()
        .process(
            serde_json::Value::Object(
                outputs
                    .into_iter()
                    .map(|(name, val)| (name, val.into()))
                    .collect(),
            ),
            format!(
                r#"
    {{ "{name}":
        {{ 
        result: job.status,
        outputs: value
        }}
    }}
        
        "#
            ),
        )
        .await
    {
        match job_result[&name]["result"].as_str() {
            Some("success") => {
                let _ = success.send_one(().into()).await;
            }
            Some("failure") => {
                let _ = failure.send_one(().into()).await;
            }
            _ => {}
        }

        let _ = result
            .send_one(Value::Data(Arc::new(Json(job_result))))
            .await;
    }
    let _ = finished.send_one(().into()).await;
}

#[mel_treatment(
    model contexts JavaScriptEngine

    input workflow_id Block<string>
    input step_id Block<string>
    input continue_on_error Block<bool>
    input spawn_completed Block<void>
    input spawn_failed Block<void>

    output step_completed Block<void>
    output step_failed Block<void>
    output step_continue Block<void>
)]
pub async fn github_set_outputs() {
    let engine = JavaScriptEngineModel::into(contexts);

    if let (Ok(workflow_id), Ok(step_id)) = (
        workflow_id
            .recv_one()
            .await
            .map(|val| GetData::<String>::try_data(val).unwrap()),
        step_id
            .recv_one()
            .await
            .map(|val| GetData::<String>::try_data(val).unwrap()),
    ) {
        let continue_on_error = continue_on_error
            .recv_one()
            .await
            .map(|val| GetData::<bool>::try_data(val).unwrap())
            .unwrap_or(false);

        let conclusion;
        let outcome;
        let outputs_contents;
        let continue_after;
        let completed;
        let failed;
        match (
            spawn_completed.recv_one().await,
            spawn_failed.recv_one().await,
        ) {
            (Ok(_), Err(_)) => {
                conclusion = serde_json::Value::String("success".to_string());
                outcome = serde_json::Value::String("success".to_string());
                continue_after = true;
                completed = true;
                failed = false;

                let mut map = serde_json::Map::new();
                if let Ok(env) = dotenvy::from_filename_iter(output_file(&workflow_id, &step_id)) {
                    for item in env {
                        if let Ok((key, val)) = item {
                            map.insert(key, serde_json::Value::String(val));
                        }
                    }
                }
                outputs_contents = serde_json::Value::Object(map);
            }
            (Err(_), Ok(_)) => {
                completed = false;
                failed = true;
                outcome = serde_json::Value::String("failure".to_string());
                if continue_on_error {
                    conclusion = serde_json::Value::String("success".to_string());
                    continue_after = true;
                } else {
                    conclusion = serde_json::Value::String("failure".to_string());
                    continue_after = false;
                }
                outputs_contents = serde_json::Value::Object(serde_json::Map::new());
            }
            _ => {
                conclusion = serde_json::Value::String("skipped".to_string());
                outcome = serde_json::Value::String("skipped".to_string());
                continue_after = false;
                completed = false;
                failed = false;
                outputs_contents = serde_json::Value::Object(serde_json::Map::new());
            }
        }
        let mut full_object = serde_json::Map::new();
        full_object.insert(
            "identifier".to_string(),
            serde_json::Value::String(step_id.clone()),
        );
        full_object.insert("conclusion".to_string(), conclusion);
        full_object.insert("outcome".to_string(), outcome);
        full_object.insert("outputs".to_string(), outputs_contents);
        let _ = engine
            .inner()
            .process(
                serde_json::Value::Object(full_object),
                r#"
                let identifier = value.identifier;
                if (value.outcome == "failure") { job.status = "failure"; }
                delete value.identifier;
                if (typeof steps !== "object") { steps = {}; }
                if (typeof steps[identifier] !== "object") { steps[identifier] = {}; }
                steps[identifier] = { ...steps[identifier], ...value };
                
                "#
                .to_string(),
            )
            .await;

        if completed {
            let _ = step_completed.send_one(().into()).await;
        }
        if failed {
            let _ = step_failed.send_one(().into()).await;
        }
        if continue_after {
            let _ = step_continue.send_one(().into()).await;
        }
    }
}

#[mel_treatment(
    input workflow_id Block<string>
    output filename Block<string>
)]
pub async fn github_get_env_files() {
    if let Ok(workflow_id) = workflow_id
        .recv_one()
        .await
        .map(|val| GetData::<String>::try_data(val).unwrap())
    {
        let _ = filename.send_one(env_file(&workflow_id).into()).await;
    }
}

fn env_file(id: &str) -> String {
    let mut file_path = std::env::temp_dir();
    file_path.push(format!("{id}.env"));
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&file_path);
    file_path.to_string_lossy().to_string()
}

fn output_file(workflow_id: &str, step_id: &str) -> String {
    let mut file_path = std::env::temp_dir();
    file_path.push(format!("{workflow_id}-{step_id}.output"));
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&file_path);
    file_path.to_string_lossy().to_string()
}

fn workspace_dir(workflow_id: &str) -> String {
    let mut file_path = std::env::temp_dir();
    file_path.push(format!("{workflow_id}-workspace"));
    let _ = std::fs::create_dir_all(&file_path);
    file_path.to_string_lossy().to_string()
}

static VAR_REGEX: OnceLock<Regex> = OnceLock::new();

fn github_regex() -> &'static Regex {
    VAR_REGEX.get_or_init(|| Regex::new(r#"\$\{\{([^}]*)\}\}"#).unwrap())
}

fn map_eval(
    map: &HashMap<String, String>,
    local_context: &serde_json::Value,
    js_engine: &JavaScriptEngineModel,
    assume: bool,
) -> HashMap<String, String> {
    let mut updated_map = HashMap::new();
    let var_replacer = VarReplacer {
        js_engine,
        local_context,
    };
    for (name, value) in map {
        let regex = github_regex();
        if assume {
            if regex.is_match(&value) {
                updated_map.insert(
                    name.clone(),
                    regex.replace_all(value, &var_replacer).to_string(),
                );
            } else {
                updated_map.insert(name.clone(), eval_js(&js_engine, &local_context, &value));
            }
        } else {
            updated_map.insert(
                name.clone(),
                regex.replace_all(value, &var_replacer).to_string(),
            );
        }
    }
    updated_map
}

#[cfg(feature = "real")]
fn eval_js(
    js_engine: &JavaScriptEngineModel,
    local_context: &serde_json::Value,
    eval: &str,
) -> String {
    // Transform the "inner value" part of evaluations
    static INNER_REGEX: OnceLock<Regex> = OnceLock::new();

    let inner_regex = INNER_REGEX.get_or_init(|| Regex::new(r#"\.([\w-]+)"#).unwrap());
    eprintln!("Pre eval: {eval}");
    let eval = inner_regex.replace_all(eval, r#"["$1"]"#);
    eprintln!("Prepared eval: {eval}");

    let eval = format!(
        r#"
{{
    let new_env = {{...(typeof env === "undefined" ? {{}} : env), ...value?.env }};
    
    {{
        let env = new_env;

        {eval}
    }}
    
}}
    "#,
    );
    async_std::task::block_on(
        js_engine
            .inner()
            .process(local_context.clone(), eval.to_string()),
    )
    .map(|res| {
        eprintln!("Eval: {eval}\nGives: {res:?}");
        res.map(|val| val.as_str().map(|s| s.to_string())).ok()
    })
    .ok()
    .flatten()
    .flatten()
    .unwrap_or_default()
}

#[cfg(feature = "mock")]
fn eval_js(
    _js_engine: &JavaScriptEngineModel,
    _local_context: &serde_json::Value,
    _eval: &str,
) -> String {
    String::new()
}

struct VarReplacer<'a> {
    pub js_engine: &'a JavaScriptEngineModel,
    pub local_context: &'a serde_json::Value,
}

impl<'a> Replacer for &VarReplacer<'a> {
    fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
        let eval = eval_js(&self.js_engine, &self.local_context, &caps[1]);
        dst.push_str(eval.as_str());
    }
}
