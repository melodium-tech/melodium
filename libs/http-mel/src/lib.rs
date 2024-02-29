#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

use melodium_core::{executive::*, *};
use melodium_macro::{mel_data, mel_function, mel_package};
use trillium::{Method, Status};

pub mod client;
pub mod server;

pub mod new_server;

#[mel_data(traits(ToString PartialEquality Equality Display))]
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct HttpMethod(Method);

impl ToString for HttpMethod {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Display for HttpMethod {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", melodium_core::executive::ToString::to_string(self))
    }
}

#[mel_data(traits(ToString PartialEquality Equality Display))]
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct HttpStatus(Status);

impl ToString for HttpStatus {
    fn to_string(&self) -> string {
        format!("{} {}", self.0 as u16, self.0.canonical_reason())
    }
}

impl Display for HttpStatus {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", melodium_core::executive::ToString::to_string(self))
    }
}

#[mel_function]
pub fn delete() -> HttpMethod {
    HttpMethod(Method::Delete)
}

#[mel_function]
pub fn get() -> HttpMethod {
    HttpMethod(Method::Get)
}

#[mel_function]
pub fn head() -> HttpMethod {
    HttpMethod(Method::Head)
}

#[mel_function]
pub fn options() -> HttpMethod {
    HttpMethod(Method::Options)
}

#[mel_function]
pub fn patch() -> HttpMethod {
    HttpMethod(Method::Patch)
}

#[mel_function]
pub fn post() -> HttpMethod {
    HttpMethod(Method::Post)
}

#[mel_function]
pub fn put() -> HttpMethod {
    HttpMethod(Method::Put)
}

#[mel_function]
pub fn trace() -> HttpMethod {
    HttpMethod(Method::Trace)
}

#[mel_function]
pub fn ok() -> HttpStatus {
    HttpStatus(Status::Ok)
}

#[mel_data(traits(ToString PartialEquality Equality Display))]
#[derive(Debug, PartialEq, Eq, Serialize)]
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
