use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::HashMap, net::IpAddr};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Access {
    pub id: Uuid,
    pub addresses: Vec<IpAddr>,
    pub port: u16,
    pub key: Uuid,
    #[serde(skip)] // Default to false
    pub disable_tls: bool,
    /// Whether plain TCP may be used with addresses other than loopback ones.
    #[serde(skip)] // Default to false
    pub allow_plain_tcp: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    pub config: Option<RequestConfig>,
    pub id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
    pub edition: Option<String>,
    pub version: String,
    pub mode: ModeRequest,
    pub max_duration: Option<u32>,
    pub memory: Option<u32>,
    pub cpu: Option<u32>,
    pub storage: Option<u32>,
    pub arch: Option<Arch>,
    /// System the worker must run, as `"os": {...}` or `"image": "..."` in the JSON.
    #[serde(flatten)]
    pub system: SystemRequest,
    pub volumes: Vec<Volume>,
    pub containers: Vec<Container>,
    pub service_containers: Vec<ServiceContainer>,
    pub tags: Vec<String>,
    pub group_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    #[serde(default)]
    pub local_exec: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestConfig {
    pub cluster_mode_preference: Option<Vec<ServiceMode>>,
    pub cluster_mode_requirement: Option<Vec<ServiceMode>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceMode {
    Shared,
    Dedicated,
    SelfManaged,
    #[serde(other, deserialize_with = "ignore_contents")]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModeRequest {
    Direct,
    Distribution,
    DirectProject { entrypoint: String, project: () },
    DistributionSecretKey { key: Uuid },
}

/// CPU architecture, named as in the OCI image specification (`GOARCH` values),
/// with `wasm32` and `wasm64` for WebAssembly.
///
/// The names Rust uses (`x86_64`, `aarch64`, `i686`, `powerpc64le`, `loongarch64`, `mipsel`...) are also read,
/// but architectures are always written with the OCI names.
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Arch {
    #[serde(alias = "x86_64")]
    Amd64,
    #[serde(alias = "aarch64")]
    Arm64,
    #[serde(
        rename = "386",
        alias = "x86",
        alias = "i686",
        alias = "i586",
        alias = "i386"
    )]
    I386,
    #[serde(alias = "armv7")]
    Arm,
    #[serde(alias = "riscv64gc")]
    Riscv64,
    #[serde(alias = "powerpc64le")]
    Ppc64le,
    #[serde(alias = "powerpc64")]
    Ppc64,
    S390x,
    #[serde(alias = "loongarch64")]
    Loong64,
    Mips,
    #[serde(alias = "mipsel")]
    Mipsle,
    Mips64,
    #[serde(alias = "mips64el")]
    Mips64le,
    #[serde(alias = "wasm")]
    Wasm32,
    Wasm64,
    #[serde(other)]
    Unknown,
}
impl Arch {
    /// Architecture of the machine this engine runs on.
    pub fn host() -> Arch {
        let little = cfg!(target_endian = "little");
        match env!("ARCH") {
            "x86_64" => Arch::Amd64,
            "aarch64" => Arch::Arm64,
            "x86" => Arch::I386,
            "arm" => Arch::Arm,
            "riscv64" => Arch::Riscv64,
            "powerpc64" if little => Arch::Ppc64le,
            "powerpc64" => Arch::Ppc64,
            "s390x" => Arch::S390x,
            "loongarch64" => Arch::Loong64,
            "mips" if little => Arch::Mipsle,
            "mips" => Arch::Mips,
            "mips64" if little => Arch::Mips64le,
            "mips64" => Arch::Mips64,
            "wasm32" => Arch::Wasm32,
            "wasm64" => Arch::Wasm64,
            _ => Arch::Unknown,
        }
    }
}

impl std::fmt::Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arch::Amd64 => write!(f, "amd64"),
            Arch::Arm64 => write!(f, "arm64"),
            Arch::I386 => write!(f, "386"),
            Arch::Arm => write!(f, "arm"),
            Arch::Riscv64 => write!(f, "riscv64"),
            Arch::Ppc64le => write!(f, "ppc64le"),
            Arch::Ppc64 => write!(f, "ppc64"),
            Arch::S390x => write!(f, "s390x"),
            Arch::Loong64 => write!(f, "loong64"),
            Arch::Mips => write!(f, "mips"),
            Arch::Mipsle => write!(f, "mipsle"),
            Arch::Mips64 => write!(f, "mips64"),
            Arch::Mips64le => write!(f, "mips64le"),
            Arch::Wasm32 => write!(f, "wasm32"),
            Arch::Wasm64 => write!(f, "wasm64"),
            _ => write!(f, "unknown"),
        }
    }
}

/// What the worker runs on: an operating system described by its characteristics,
/// or one exact machine image. Both cannot be given at once.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum System {
    /// Operating system the worker must run.
    Os(Os),
    /// Name of the machine image the worker must boot from, resolved by whatever spawns the worker (never a container image).
    Image(String),
}

/// The system need of a request, `None` when the request has no specific need.
///
/// It is written as an `os` or an `image` field of the request, and a request with both is refused.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "SystemFields", into = "SystemFields")]
pub struct SystemRequest(pub Option<System>);

#[derive(Serialize, Deserialize)]
struct SystemFields {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    os: Option<Os>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    image: Option<String>,
}

impl TryFrom<SystemFields> for SystemRequest {
    type Error = String;
    fn try_from(fields: SystemFields) -> Result<Self, Self::Error> {
        match (fields.os, fields.image) {
            (None, None) => Ok(SystemRequest(None)),
            (Some(os), None) => Ok(SystemRequest(Some(System::Os(os)))),
            (None, Some(image)) => Ok(SystemRequest(Some(System::Image(image)))),
            (Some(_), Some(_)) => Err(
                "an operating system and a machine image cannot be requested together".to_string(),
            ),
        }
    }
}

impl From<SystemRequest> for SystemFields {
    fn from(request: SystemRequest) -> Self {
        match request.0 {
            None => SystemFields {
                os: None,
                image: None,
            },
            Some(System::Os(os)) => SystemFields {
                os: Some(os),
                image: None,
            },
            Some(System::Image(image)) => SystemFields {
                os: None,
                image: Some(image),
            },
        }
    }
}

impl std::fmt::Display for System {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            System::Os(os) => write!(f, "{os}"),
            System::Image(image) => write!(f, "image {image}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Os {
    pub family: OsFamily,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distribution: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl std::fmt::Display for Os {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.family)?;
        if let Some(distribution) = &self.distribution {
            write!(f, " {distribution}")?;
        }
        if let Some(version) = &self.version {
            write!(f, " {version}")?;
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OsFamily {
    Linux,
    Windows,
    Macos,
    Freebsd,
    Openbsd,
    Netbsd,
    Solaris,
    Illumos,
    Aix,
    Ibmi,
    Zos,
    #[serde(other)]
    Unknown,
}

impl std::fmt::Display for OsFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OsFamily::Linux => write!(f, "linux"),
            OsFamily::Windows => write!(f, "windows"),
            OsFamily::Macos => write!(f, "macos"),
            OsFamily::Freebsd => write!(f, "freebsd"),
            OsFamily::Openbsd => write!(f, "openbsd"),
            OsFamily::Netbsd => write!(f, "netbsd"),
            OsFamily::Solaris => write!(f, "solaris"),
            OsFamily::Illumos => write!(f, "illumos"),
            OsFamily::Aix => write!(f, "aix"),
            OsFamily::Ibmi => write!(f, "ibmi"),
            OsFamily::Zos => write!(f, "zos"),
            _ => write!(f, "unknown"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Volume {
    pub name: String,
    pub storage: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Container {
    pub name: String,
    pub image: String,
    pub pull_secret: String,
    pub memory: u32,
    pub cpu: u32,
    pub storage: u32,
    /// Architecture the container requires, none meaning the one of the machine the worker runs on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch: Option<Arch>,
    pub mounts: Vec<VolumeMount>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceContainer {
    pub name: String,
    pub image: String,
    pub pull_secret: String,
    pub memory: u32,
    pub cpu: u32,
    pub storage: u32,
    /// Architecture the container requires, none meaning the one of the machine the worker runs on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arch: Option<Arch>,
    pub mounts: Vec<VolumeMount>,
    pub env: HashMap<String, String>,
    pub command: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VolumeMount {
    pub name: String,
    pub mount_point: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    Ok(Uuid),
    Error(Vec<String>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistributionResponse {
    Started(Option<Access>),
    Error(Vec<String>),
}

fn ignore_contents<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
{
    // Ignore any content at this part of the json structure
    let _ = deserializer.deserialize_ignored_any(serde::de::IgnoredAny);

    // Return unit as our 'Unknown' variant has no args
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistributionResult {
    Success(Option<Vec<String>>),
    Failure(Option<Vec<String>>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalEnd {
    pub run_id: Uuid,
    pub result: DistributionResult,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalLaunched {
    pub run_id: Uuid,
    pub response: DistributionResponse,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> Request {
        Request {
            config: None,
            id: None,
            organization_id: None,
            edition: None,
            version: "0.11.0".to_string(),
            mode: ModeRequest::Direct,
            max_duration: None,
            memory: None,
            cpu: None,
            storage: None,
            arch: None,
            system: SystemRequest(None),
            volumes: vec![],
            containers: vec![],
            service_containers: vec![],
            tags: vec![],
            group_id: None,
            parent_id: None,
            local_exec: false,
        }
    }

    #[test]
    fn os_serialization() {
        let mut request = request();
        request.system = SystemRequest(Some(System::Os(Os {
            family: OsFamily::Linux,
            distribution: Some("nixos".to_string()),
            version: Some("25.05".to_string()),
        })));

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json["os"],
            serde_json::json!({"family": "linux", "distribution": "nixos", "version": "25.05"})
        );
        assert!(json.get("image").is_none());
        assert!(json.get("system").is_none());
    }

    #[test]
    fn image_serialization() {
        let mut request = request();
        request.system = SystemRequest(Some(System::Image("nixos-ci".to_string())));

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["image"], "nixos-ci");
        assert!(json.get("os").is_none());
    }

    #[test]
    fn system_omitted_when_unset() {
        let json = serde_json::to_value(request()).unwrap();
        assert!(json.get("os").is_none());
        assert!(json.get("image").is_none());
    }

    #[test]
    fn system_roundtrip() {
        for system in [
            None,
            Some(System::Image("nixos-ci".to_string())),
            Some(System::Os(Os {
                family: OsFamily::Windows,
                distribution: None,
                version: Some("2022".to_string()),
            })),
        ] {
            let mut original = request();
            original.system = SystemRequest(system.clone());
            let back: Request =
                serde_json::from_value(serde_json::to_value(&original).unwrap()).unwrap();
            assert_eq!(back.system.0, system);
        }
    }

    #[test]
    fn os_and_image_together_refused() {
        let mut json = serde_json::to_value(request()).unwrap();
        json["os"] = serde_json::json!({"family": "linux"});
        json["image"] = serde_json::json!("nixos-ci");
        assert!(serde_json::from_value::<Request>(json).is_err());
    }

    #[test]
    fn unknown_os_family() {
        let os: Os = serde_json::from_str(r#"{"family": "plan9"}"#).unwrap();
        assert_eq!(os.family, OsFamily::Unknown);
        assert_eq!(os.distribution, None);
    }

    #[test]
    fn arch_rust_aliases() {
        for (alias, arch) in [
            ("x86_64", Arch::Amd64),
            ("aarch64", Arch::Arm64),
            ("x86", Arch::I386),
            ("i686", Arch::I386),
            ("armv7", Arch::Arm),
            ("riscv64gc", Arch::Riscv64),
            ("powerpc64le", Arch::Ppc64le),
            ("powerpc64", Arch::Ppc64),
            ("loongarch64", Arch::Loong64),
            ("mipsel", Arch::Mipsle),
            ("mips64el", Arch::Mips64le),
            ("wasm", Arch::Wasm32),
        ] {
            assert_eq!(
                serde_json::from_value::<Arch>(serde_json::json!(alias)).unwrap(),
                arch
            );
        }
        // Written with the OCI name.
        assert_eq!(
            serde_json::to_value(Arch::Amd64).unwrap(),
            serde_json::json!("amd64")
        );
    }

    #[test]
    fn arch_names() {
        for (arch, name) in [
            (Arch::Amd64, "amd64"),
            (Arch::Arm64, "arm64"),
            (Arch::I386, "386"),
            (Arch::Arm, "arm"),
            (Arch::Riscv64, "riscv64"),
            (Arch::Ppc64le, "ppc64le"),
            (Arch::Ppc64, "ppc64"),
            (Arch::S390x, "s390x"),
            (Arch::Loong64, "loong64"),
            (Arch::Mips, "mips"),
            (Arch::Mipsle, "mipsle"),
            (Arch::Mips64, "mips64"),
            (Arch::Mips64le, "mips64le"),
            (Arch::Wasm32, "wasm32"),
            (Arch::Wasm64, "wasm64"),
        ] {
            assert_eq!(arch.to_string(), name);
            assert_eq!(serde_json::to_value(arch).unwrap(), serde_json::json!(name));
            assert_eq!(
                serde_json::from_value::<Arch>(serde_json::json!(name)).unwrap(),
                arch
            );
        }
        assert_eq!(
            serde_json::from_value::<Arch>(serde_json::json!("sparc64")).unwrap(),
            Arch::Unknown
        );
    }
}
