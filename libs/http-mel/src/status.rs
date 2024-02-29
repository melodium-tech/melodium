use melodium_core::{executive::*, *};
use melodium_macro::{mel_data, mel_function,};
use trillium::Status;

#[mel_data(traits(ToString PartialEquality Equality Display))]
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct HttpStatus(pub Status);

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
pub fn status(code: u16) -> Option<HttpStatus> {
    match Status::try_from(code) {
        Ok(status) => Some(HttpStatus(status)),
        Err(_) => None,
    }
}

#[mel_function]
pub fn ok() -> HttpStatus {
    HttpStatus(Status::Ok)
}

#[mel_function]
pub fn moved_permanently() -> HttpStatus {
    HttpStatus(Status::MovedPermanently)
}

#[mel_function]
pub fn temporary_redirect() -> HttpStatus {
    HttpStatus(Status::TemporaryRedirect)
}

#[mel_function]
pub fn permanent_redirect() -> HttpStatus {
    HttpStatus(Status::PermanentRedirect)
}

#[mel_function]
pub fn forbidden() -> HttpStatus {
    HttpStatus(Status::Forbidden)
}

#[mel_function]
pub fn not_found() -> HttpStatus {
    HttpStatus(Status::NotFound)
}