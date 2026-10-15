use crate::api;
use melodium_core::{executive::*, *};
use melodium_macro::{mel_data, mel_function};

/// What a work request needs its worker to run on.
///
/// Either an operating system, described by its family and optionally its distribution and version,
/// or one exact machine image. Both cannot be requested at once, and a request may give none.
///
/// `ToString` and `Display` return the operating system family followed by the distribution and the version when given
/// (e.g. `"linux"`, `"linux nixos"`, `"linux ubuntu 24.04"`, `"windows 2022"`), or `"image "` followed by the image name.
#[mel_data(
    traits (PartialEquality Serialize Deserialize Display ToString)
)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct System(pub api::System);

impl ToString for System {
    fn to_string(&self) -> String {
        format!("{}", self.0)
    }
}

impl Display for System {
    fn display(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", self.0)
    }
}

fn os(family: api::OsFamily, distribution: Option<String>, version: Option<String>) -> System {
    System(api::System::Os(api::Os {
        family,
        distribution,
        version,
    }))
}

/// Return the `linux` operating system, of any distribution and version.
///
/// Use as the `system` parameter when requesting workers.
#[mel_function]
pub fn linux() -> System {
    os(api::OsFamily::Linux, None, None)
}

/// Return a `linux` operating system of the given distribution, of any version.
///
/// The distribution is the `ID` field of `/etc/os-release` (e.g. `"ubuntu"`, `"debian"`, `"rhel"`, `"fedora"`, `"alpine"`, `"nixos"`),
/// compared case-insensitively.
#[mel_function]
pub fn linux_distribution(name: string) -> System {
    os(api::OsFamily::Linux, Some(name), None)
}

/// Return a `linux` operating system of the given distribution and version.
///
/// The version is the `VERSION_ID` field of `/etc/os-release` (e.g. `"24.04"` for `"ubuntu"`),
/// and a more precise version satisfies a less precise request (`"24.04.1"` satisfies `"24.04"`).
#[mel_function]
pub fn linux_distribution_version(name: string, version: string) -> System {
    os(api::OsFamily::Linux, Some(name), Some(version))
}

/// Return the `windows` operating system, of any version.
#[mel_function]
pub fn windows() -> System {
    os(api::OsFamily::Windows, None, None)
}

/// Return a `windows` operating system of the given version (e.g. `"2022"`).
#[mel_function]
pub fn windows_version(version: string) -> System {
    os(api::OsFamily::Windows, None, Some(version))
}

/// Return the `macos` operating system, of any version.
#[mel_function]
pub fn macos() -> System {
    os(api::OsFamily::Macos, None, None)
}

/// Return a `macos` operating system of the given major version (e.g. `"15"`).
#[mel_function]
pub fn macos_version(version: string) -> System {
    os(api::OsFamily::Macos, None, Some(version))
}

/// Return the `freebsd` operating system (FreeBSD), of any version.
#[mel_function]
pub fn freebsd() -> System {
    os(api::OsFamily::Freebsd, None, None)
}

/// Return a `freebsd` operating system (FreeBSD) of the given version (e.g. `"14.2"`).
#[mel_function]
pub fn freebsd_version(version: string) -> System {
    os(api::OsFamily::Freebsd, None, Some(version))
}

/// Return the `openbsd` operating system (OpenBSD), of any version.
#[mel_function]
pub fn openbsd() -> System {
    os(api::OsFamily::Openbsd, None, None)
}

/// Return a `openbsd` operating system (OpenBSD) of the given version (e.g. `"7.6"`).
#[mel_function]
pub fn openbsd_version(version: string) -> System {
    os(api::OsFamily::Openbsd, None, Some(version))
}

/// Return the `netbsd` operating system (NetBSD), of any version.
#[mel_function]
pub fn netbsd() -> System {
    os(api::OsFamily::Netbsd, None, None)
}

/// Return a `netbsd` operating system (NetBSD) of the given version (e.g. `"10.0"`).
#[mel_function]
pub fn netbsd_version(version: string) -> System {
    os(api::OsFamily::Netbsd, None, Some(version))
}

/// Return the `solaris` operating system (Oracle Solaris), of any version.
#[mel_function]
pub fn solaris() -> System {
    os(api::OsFamily::Solaris, None, None)
}

/// Return a `solaris` operating system (Oracle Solaris) of the given version (e.g. `"11.4"`).
#[mel_function]
pub fn solaris_version(version: string) -> System {
    os(api::OsFamily::Solaris, None, Some(version))
}

/// Return the `illumos` operating system (illumos (OmniOS, OpenIndiana, SmartOS, ...)), of any version.
#[mel_function]
pub fn illumos() -> System {
    os(api::OsFamily::Illumos, None, None)
}

/// Return a `illumos` operating system (illumos (OmniOS, OpenIndiana, SmartOS, ...)) of the given version (e.g. `"151050"`).
#[mel_function]
pub fn illumos_version(version: string) -> System {
    os(api::OsFamily::Illumos, None, Some(version))
}

/// Return the `aix` operating system (IBM AIX), of any version.
#[mel_function]
pub fn aix() -> System {
    os(api::OsFamily::Aix, None, None)
}

/// Return a `aix` operating system (IBM AIX) of the given version (e.g. `"7.3"`).
#[mel_function]
pub fn aix_version(version: string) -> System {
    os(api::OsFamily::Aix, None, Some(version))
}

/// Return the `ibmi` operating system (IBM i), of any version.
#[mel_function]
pub fn ibmi() -> System {
    os(api::OsFamily::Ibmi, None, None)
}

/// Return a `ibmi` operating system (IBM i) of the given version (e.g. `"7.5"`).
#[mel_function]
pub fn ibmi_version(version: string) -> System {
    os(api::OsFamily::Ibmi, None, Some(version))
}

/// Return the `zos` operating system (IBM z/OS), of any version.
#[mel_function]
pub fn zos() -> System {
    os(api::OsFamily::Zos, None, None)
}

/// Return a `zos` operating system (IBM z/OS) of the given version (e.g. `"3.1"`).
#[mel_function]
pub fn zos_version(version: string) -> System {
    os(api::OsFamily::Zos, None, Some(version))
}

/// Return one exact machine image.
///
/// `name` is the name of the image the worker boots from. It is the name of an image alias defined for the organization,
/// or a provider reference prefixed by the provider (`"scaleway:..."`, `"aws:..."`, `"gcp:..."`, `"azure:..."`, `"oci:..."`).
/// The operating system and architecture are those of the image.
#[mel_function]
pub fn image(name: string) -> System {
    System(api::System::Image(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use api::OsFamily::*;

    #[test]
    fn display() {
        assert_eq!(format!("{}", os(Linux, None, None).0), "linux");
        assert_eq!(
            format!("{}", os(Linux, Some("nixos".into()), None).0),
            "linux nixos"
        );
        assert_eq!(
            format!(
                "{}",
                os(Linux, Some("ubuntu".into()), Some("24.04".into())).0
            ),
            "linux ubuntu 24.04"
        );
        assert_eq!(
            format!("{}", os(Windows, None, Some("2022".into())).0),
            "windows 2022"
        );
        assert_eq!(format!("{}", os(Macos, None, None).0), "macos");
        assert_eq!(
            format!("{}", os(Freebsd, None, Some("14.2".into())).0),
            "freebsd 14.2"
        );
        assert_eq!(
            format!("{}", System(api::System::Image("nixos-ci".into())).0),
            "image nixos-ci"
        );
    }
}
