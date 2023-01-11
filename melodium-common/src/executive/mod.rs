
//! Executive elements.
//! 
//! This module contains essentially traits and very basic concrete types.
//! The concrete implementations are provided by engine, utilities, or core implementation from other Mélodium crates, and not aimed to be brought by user.
//! 

pub mod context;
pub mod future;
pub mod input;
pub mod model;
pub mod output;
pub mod result_status;
pub mod transmission;
pub mod treatment;
pub mod value;
pub mod world;

pub use context::Context;
pub use future::ContinuousFuture;
pub use future::TrackFuture;
pub use input::Input;
pub use model::{Model, ModelId};
pub use output::Output;
pub use result_status::ResultStatus;
pub use transmission::{RecvResult, SendResult, TransmissionError};
pub use treatment::Treatment;
pub use value::Value;
pub use world::{TrackId, World};
