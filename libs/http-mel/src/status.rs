use melodium_core::{executive::*, *};
use melodium_macro::{mel_data, mel_function};
use trillium::Status;

/// HTTP response status code.
///
/// `ToString` and `Display` return the numeric code followed by the canonical reason phrase,
/// e.g. `"200 OK"` or `"404 Not Found"`.
#[mel_data(traits(ToString PartialEquality Equality Display))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

/// Parse an HTTP status from its numeric code.
///
/// Returns `None` if `code` does not correspond to a recognised HTTP status.
#[mel_function]
pub fn status(code: u16) -> Option<HttpStatus> {
    match Status::try_from(code) {
        Ok(status) => Some(HttpStatus(status)),
        Err(_) => None,
    }
}

/// Return HTTP status `200 OK`.
#[mel_function]
pub fn ok() -> HttpStatus {
    HttpStatus(Status::Ok)
}

/// Return HTTP status `301 Moved Permanently`.
#[mel_function]
pub fn moved_permanently() -> HttpStatus {
    HttpStatus(Status::MovedPermanently)
}

/// Return HTTP status `307 Temporary Redirect`.
#[mel_function]
pub fn temporary_redirect() -> HttpStatus {
    HttpStatus(Status::TemporaryRedirect)
}

/// Return HTTP status `308 Permanent Redirect`.
#[mel_function]
pub fn permanent_redirect() -> HttpStatus {
    HttpStatus(Status::PermanentRedirect)
}

/// Return HTTP status `403 Forbidden`.
#[mel_function]
pub fn forbidden() -> HttpStatus {
    HttpStatus(Status::Forbidden)
}

/// Return HTTP status `404 Not Found`.
#[mel_function]
pub fn not_found() -> HttpStatus {
    HttpStatus(Status::NotFound)
}
