//! Executive elements.
//!
//! This module contains essentially traits and very basic concrete types.
//! The concrete implementations are provided by engine, utilities, or core implementation from other Mélodium crates, and not aimed to be brought by user.
//!

mod context;
mod data;
mod data_traits;
mod future;
mod input;
mod model;
mod output;
mod result_status;
mod transmission;
mod treatment;
mod value;
mod world;

pub use context::Context;
pub use data::Data;
pub use data_traits::DataTrait;
pub use future::ContinuousFuture;
pub use future::TrackFuture;
pub use input::Input;
pub use model::{Model, ModelId};
pub use output::{Output, Outputs};
pub use result_status::ResultStatus;
pub use transmission::{RecvResult, SendResult, TransmissionError, TransmissionValue};
pub use treatment::Treatment;
pub use value::{GetData, Value};
pub use world::{DirectCreationCallback, TrackCreationCallback, TrackId, World};
