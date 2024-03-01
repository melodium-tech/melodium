#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_core::{executive::*, *};
use melodium_macro::{mel_data, mel_function, mel_package};

pub mod client;
pub mod method;
pub mod server;
pub mod status;

pub mod new_client;
pub mod new_server;

#[mel_data(traits(ToString PartialEquality Equality Display))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Url(String);

impl ToString for Url {
    fn to_string(&self) -> string {
        self.0.clone()
    }
}

impl Display for Url {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", melodium_core::DataTrait::to_string(self))
    }
}

#[mel_function]
pub fn url(url: string) -> Url {
    Url(url)
}

mel_package!();
