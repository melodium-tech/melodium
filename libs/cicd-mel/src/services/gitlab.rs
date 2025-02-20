use melodium_core::{executive::*, *};
use melodium_macro::{mel_data, mel_function};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum State {
    Pending,
    Running,
    Success,
    Failed,
    Canceled,
    Skipped,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Pending => write!(f, "pending"),
            State::Running => write!(f, "running"),
            State::Success => write!(f, "success"),
            State::Failed => write!(f, "failed"),
            State::Canceled => write!(f, "canceled"),
            State::Skipped => write!(f, "skipped"),
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
pub fn running() -> StepState {
    StepState(State::Running)
}

#[mel_function]
pub fn success() -> StepState {
    StepState(State::Success)
}

#[mel_function]
pub fn failed() -> StepState {
    StepState(State::Failed)
}

#[mel_function]
pub fn canceled() -> StepState {
    StepState(State::Canceled)
}

#[mel_function]
pub fn skipped() -> StepState {
    StepState(State::Skipped)
}
