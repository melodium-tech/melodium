use std::{
    collections::HashMap,
    sync::OnceLock,
};
use javascript_mel::*;
use melodium_core::{executive::*, *};
use melodium_macro::{mel_data, mel_function, mel_treatment};
use regex::{Captures, Regex, Replacer};
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
    input trigger Block<void>
    output evaluated Block<StringMap>
)]
pub async fn githubEval(map: StringMap) {
    let engine = JavaScriptEngineModel::into(engine);

    if let Ok(_) = trigger.recv_one().await {
        let mut updated_map = HashMap::new();
        let var_replacer = VarReplacer { js_engine: &engine };
        for (name, value) in &map.map {
            let regex = github_regex();
            updated_map.insert(name.clone(), regex.replace_all(value, &var_replacer));
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
