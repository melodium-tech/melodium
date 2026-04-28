use melodium_core::{executive::*, *};
use melodium_macro::{mel_data, mel_function};
use trillium::Method;

/// HTTP request method.
///
/// Implements `ToString` and `Display`, which return the standard uppercase method name
/// (e.g. `"GET"`, `"POST"`).
#[mel_data(traits(ToString PartialEquality Equality Display))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HttpMethod(pub Method);

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

/// Parse an HTTP method by name.
///
/// Returns `None` if `name` is not a recognised HTTP method string.
#[mel_function]
pub fn method(name: string) -> Option<HttpMethod> {
    match Method::try_from(name.as_str()) {
        Ok(method) => Some(HttpMethod(method)),
        Err(_) => None,
    }
}

/// Return the `DELETE` HTTP method.
#[mel_function]
pub fn delete() -> HttpMethod {
    HttpMethod(Method::Delete)
}

/// Return the `GET` HTTP method.
#[mel_function]
pub fn get() -> HttpMethod {
    HttpMethod(Method::Get)
}

/// Return the `HEAD` HTTP method.
#[mel_function]
pub fn head() -> HttpMethod {
    HttpMethod(Method::Head)
}

/// Return the `OPTIONS` HTTP method.
#[mel_function]
pub fn options() -> HttpMethod {
    HttpMethod(Method::Options)
}

/// Return the `PATCH` HTTP method.
#[mel_function]
pub fn patch() -> HttpMethod {
    HttpMethod(Method::Patch)
}

/// Return the `POST` HTTP method.
#[mel_function]
pub fn post() -> HttpMethod {
    HttpMethod(Method::Post)
}

/// Return the `PUT` HTTP method.
#[mel_function]
pub fn put() -> HttpMethod {
    HttpMethod(Method::Put)
}

/// Return the `TRACE` HTTP method.
#[mel_function]
pub fn trace() -> HttpMethod {
    HttpMethod(Method::Trace)
}
