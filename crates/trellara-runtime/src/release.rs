use serde::{Deserialize, Serialize};

pub const RELEASE_CONTRACT_VERSION: u16 = 1;
pub const SUPPORTED_EXTERNAL_POSTGRES_MAJORS: [u16; 3] = [16, 17, 18];
pub const SUPPORTED_NATIVE_POSTGRES_MAJORS: [u16; 2] = [17, 18];
pub const RELEASE_ARCHITECTURES: [ReleaseArchitecture; 2] =
    [ReleaseArchitecture::Amd64, ReleaseArchitecture::Arm64];

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseComponent {
    Cli,
    Relay,
    Applier,
    NativePostgresExtension,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseOperatingSystem {
    Linux,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseArchitecture {
    Amd64,
    Arm64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReleasePackageFormat {
    TarGz,
    Deb,
    Rpm,
    OciImage,
}

#[must_use]
pub fn supports_external_postgres_major(major: u16) -> bool {
    SUPPORTED_EXTERNAL_POSTGRES_MAJORS.contains(&major)
}

#[must_use]
pub fn supports_native_postgres_major(major: u16) -> bool {
    SUPPORTED_NATIVE_POSTGRES_MAJORS.contains(&major)
}
