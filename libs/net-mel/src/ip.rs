use melodium_core::{executive::*, *};
use melodium_macro::{check, mel_data, mel_function, mel_treatment};
use std::str::FromStr;
use std::sync::Arc;

/// IP data.
///
/// `Ip` data type contains a valid IP v4 or v6.
///
/// ℹ️ _Valid IP_ means the data contained makes sense as IP, not that it is reacheable.
#[mel_data(traits(ToString TryToString Display))]
#[derive(Debug, Clone, Serialize)]
pub struct Ip(pub std::net::IpAddr);

impl ToString for Ip {
    fn to_string(&self) -> string {
        self.0.to_string()
    }
}

impl TryToString for Ip {
    fn try_to_string(&self) -> Option<string> {
        Some(self.0.to_string())
    }
}

impl Display for Ip {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", &self.0)
    }
}

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

/// Makes IPv4 generic.
#[mel_function]
pub fn from_ipv4(ipv4: Ipv4) -> Ip {
    Ip(std::net::IpAddr::V4(ipv4.0))
}

/// Makes IPv6 generic.
#[mel_function]
pub fn from_ipv6(ipv6: Ipv6) -> Ip {
    Ip(std::net::IpAddr::V6(ipv6.0))
}

/// Turn stream of IPv4 into generic IP
#[mel_treatment(
    input ipv4 Stream<Ipv4>
    output ip Stream<Ip>
)]
pub async fn from_ipv4() {
    while let Ok(ips) = ipv4
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            ip.send_many(TransmissionValue::Other(
                ips.into_iter()
                    .map(|ip| Value::Data(Arc::new(Ip(std::net::IpAddr::V4(
                        GetData::<Arc<dyn Data>>::try_data(ip)
                            .unwrap()
                            .downcast_arc::<Ipv4>()
                            .unwrap()
                            .0
                    )))))
                    .collect()
            ))
            .await
        )
    }
}

/// Turn stream of IPv6 into generic IP
#[mel_treatment(
    input ipv6 Stream<Ipv6>
    output ip Stream<Ip>
)]
pub async fn from_ipv6() {
    while let Ok(ips) = ipv6
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            ip.send_many(TransmissionValue::Other(
                ips.into_iter()
                    .map(|ip| Value::Data(Arc::new(Ip(std::net::IpAddr::V6(
                        GetData::<Arc<dyn Data>>::try_data(ip)
                            .unwrap()
                            .downcast_arc::<Ipv6>()
                            .unwrap()
                            .0
                    )))))
                    .collect()
            ))
            .await
        )
    }
}

/// Get the IPv4 of generic IP.
#[mel_function]
pub fn as_ipv4(ip: Ip) -> Option<Ipv4> {
    if let std::net::IpAddr::V4(ip) = ip.0 {
        Some(Ipv4(ip))
    } else {
        None
    }
}

/// Get the IPv6 of generic IP.
#[mel_function]
pub fn as_ipv6(ip: Ip) -> Option<Ipv6> {
    if let std::net::IpAddr::V6(ip) = ip.0 {
        Some(Ipv6(ip))
    } else {
        None
    }
}

/// Get the IPv4 from generic IP stream
#[mel_treatment(
    input ip Stream<Ip>
    output ipv4 Stream<Option<Ipv4>>
)]
pub async fn as_ipv4() {
    while let Ok(ips) = ip
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            ipv4.send_many(TransmissionValue::Other(
                ips.into_iter()
                    .map(|ip| Value::Option(
                        match GetData::<Arc<dyn Data>>::try_data(ip)
                            .unwrap()
                            .downcast_arc::<Ip>()
                            .unwrap()
                            .0
                        {
                            std::net::IpAddr::V4(ip) =>
                                Some(Box::new(Value::Data(Arc::new(Ipv4(ip))))),
                            std::net::IpAddr::V6(_) => None,
                        }
                    ))
                    .collect()
            ))
            .await
        )
    }
}

/// Get the IPv6 from generic IP stream
#[mel_treatment(
    input ip Stream<Ip>
    output ipv6 Stream<Option<Ipv6>>
)]
pub async fn as_ipv6() {
    while let Ok(ips) = ip
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            ipv6.send_many(TransmissionValue::Other(
                ips.into_iter()
                    .map(|ip| Value::Option(
                        match GetData::<Arc<dyn Data>>::try_data(ip)
                            .unwrap()
                            .downcast_arc::<Ip>()
                            .unwrap()
                            .0
                        {
                            std::net::IpAddr::V4(_) => None,
                            std::net::IpAddr::V6(ip) =>
                                Some(Box::new(Value::Data(Arc::new(Ipv6(ip))))),
                        }
                    ))
                    .collect()
            ))
            .await
        )
    }
}

/// Tells if generic IP is IPv4.
#[mel_function]
pub fn is_ipv4(ip: Ip) -> bool {
    ip.0.is_ipv4()
}

/// Tells if generic IP is IPv6.
#[mel_function]
pub fn is_ipv6(ip: Ip) -> bool {
    ip.0.is_ipv6()
}

/// Tells for IP in stream if they are v4
#[mel_treatment(
    input ip Stream<Ip>
    output ipv4 Stream<bool>
)]
pub async fn is_ipv4() {
    while let Ok(ips) = ip
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            ipv4.send_many(TransmissionValue::Bool(
                ips.into_iter()
                    .map(|ip| GetData::<Arc<dyn Data>>::try_data(ip)
                        .unwrap()
                        .downcast_arc::<Ip>()
                        .unwrap()
                        .0
                        .is_ipv4())
                    .collect()
            ))
            .await
        )
    }
}

/// Tells for IP in stream if they are v6
#[mel_treatment(
    input ip Stream<Ip>
    output ipv6 Stream<bool>
)]
pub async fn is_ipv6() {
    while let Ok(ips) = ip
        .recv_many()
        .await
        .map(|values| Into::<VecDeque<Value>>::into(values))
    {
        check!(
            ipv6.send_many(TransmissionValue::Bool(
                ips.into_iter()
                    .map(|ip| GetData::<Arc<dyn Data>>::try_data(ip)
                        .unwrap()
                        .downcast_arc::<Ip>()
                        .unwrap()
                        .0
                        .is_ipv6())
                    .collect()
            ))
            .await
        )
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
