use melodium_core::{executive::*, *};
use melodium_macro::{mel_data, mel_function};
use trillium::Method;

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

#[mel_function]
pub fn method(name: string) -> Option<HttpMethod> {
    match Method::try_from(name.as_str()) {
        Ok(method) => Some(HttpMethod(method)),
        Err(_) => None,
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
