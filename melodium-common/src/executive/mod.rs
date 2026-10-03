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
mod log;
mod model;
mod output;
mod result_status;
mod secret;
mod transmission;
mod treatment;
mod value;
mod wipe;
mod world;

pub use context::Context;
pub use data::Data;
pub use data_traits::DataTrait;
pub use future::ContinuousFuture;
pub use future::TrackFuture;
pub use input::{Input, InputExt};
pub use log::{Level, Log};
pub use model::{Model, ModelId};
pub use output::{Output, OutputExt, Outputs};
pub use result_status::ResultStatus;
pub use secret::{
    with_secret_wire, SecretAccess, SecretAudit, SecretAuditOutcome, SecretDerivation, SecretError,
    SecretSource, SecretWire,
};
pub use transmission::{RecvResult, SendResult, TransmissionError, TransmissionValue};
pub use treatment::Treatment;
pub use value::{
    GetData, PackedArray, Secret, SecretId, SecretOrigin, SecretPolicy, SecretReveal,
    SecretTransfer, SecretTransmission, Value,
};
pub use wipe::{count_wiped, register_wipe, wipe_all, wipe_value, wiped_count, Wipe};
pub use world::{DirectCreationCallback, TrackCreationCallback, TrackId, World};
