use melodium_core::{executive::*, *};
use melodium_macro::{check, mel_data, mel_function, mel_treatment};
use std::str::FromStr;
use std::sync::Arc;

/// IPv4 data.
///
/// `Ipv4` data type contains a valid IP v4.
///
/// ℹ️ _Valid IP v4_ means the data contained makes sense as IP, not that it is reacheable.
#[mel_data(traits(ToString TryToString Display))]
#[derive(Debug, Clone, Serialize)]
pub struct Ipv4(pub std::net::Ipv4Addr);

impl ToString for Ipv4 {
    fn to_string(&self) -> string {
        self.0.to_string()
    }
}

impl TryToString for Ipv4 {
    fn try_to_string(&self) -> Option<string> {
        Some(self.0.to_string())
    }
}

impl Display for Ipv4 {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", &self.0)
    }
}

/// Creates new IPv4.
#[mel_function]
pub fn ipv4(a: u8, b: u8, c: u8, d: u8) -> Ipv4 {
    Ipv4(std::net::Ipv4Addr::new(a, b, c, d))
}

/// Parse string into IPv4.
#[mel_function]
pub fn to_ipv4(text: string) -> Option<Ipv4> {
    std::net::Ipv4Addr::from_str(&text).ok().map(|ip| Ipv4(ip))
}

/// Parse string into IPv4.
///
/// `ipv4` contains some `Ipv4` if input `text` contains valid ip, else none.
#[mel_treatment(
    input text Stream<string>
    output ipv4 Stream<Option<Ipv4>>
)]
pub async fn to_ipv4() {
    while let Ok(text) = text
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
        check!(
            ipv4.send_many(TransmissionValue::Other(
                text.iter()
                    .map(|t| Value::Option(
                        std::net::Ipv4Addr::from_str(t)
                            .ok()
                            .map(|ip| Box::new(Value::Data(Arc::new(Ipv4(ip)))))
                    ))
                    .collect()
            ))
            .await
        )
    }
}

/// Return IPv4 localhost.
#[mel_function]
pub fn localhost_ipv4() -> Ipv4 {
    Ipv4(std::net::Ipv4Addr::LOCALHOST)
}

/// IPv6 data.
///
/// `Ipv6` data type contains a valid IP v6.
///
/// ℹ️ _Valid IP v6_ means the data contained makes sense as IP, not that it is reacheable.
#[mel_data(traits(ToString TryToString Display))]
#[derive(Debug, Clone, Serialize)]
pub struct Ipv6(pub std::net::Ipv6Addr);

impl ToString for Ipv6 {
    fn to_string(&self) -> string {
        self.0.to_string()
    }
}

impl TryToString for Ipv6 {
    fn try_to_string(&self) -> Option<string> {
        Some(self.0.to_string())
    }
}

impl Display for Ipv6 {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", &self.0)
    }
}

/// Creates new IPv6.
#[mel_function]
pub fn ipv6(a: u16, b: u16, c: u16, d: u16, e: u16, f: u16, g: u16, h: u16) -> Ipv6 {
    Ipv6(std::net::Ipv6Addr::new(a, b, c, d, e, f, g, h))
}

/// Parse string into IPv6.
#[mel_function]
pub fn to_ipv6(text: string) -> Option<Ipv6> {
    std::net::Ipv6Addr::from_str(&text).ok().map(|ip| Ipv6(ip))
}

/// Parse string into IPv6.
///
/// `ipv6` contains some `Ipv6` if input `text` contains valid ip, else none.
#[mel_treatment(
    input text Stream<string>
    output ipv6 Stream<Option<Ipv6>>
)]
pub async fn to_ipv6() {
    while let Ok(text) = text
        .recv_many()
        .await
        .map(|values| TryInto::<Vec<string>>::try_into(values).unwrap())
    {
        check!(
            ipv6.send_many(TransmissionValue::Other(
                text.iter()
                    .map(|t| Value::Option(
                        std::net::Ipv6Addr::from_str(t)
                            .ok()
                            .map(|ip| Box::new(Value::Data(Arc::new(Ipv6(ip)))))
                    ))
                    .collect()
            ))
            .await
        )
    }
}

/// Return IPv6 localhost.
#[mel_function]
pub fn localhost_ipv6() -> Ipv6 {
    Ipv6(std::net::Ipv6Addr::LOCALHOST)
}
