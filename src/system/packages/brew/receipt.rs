//! Homebrew-compatible receipt schemas and serialization.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;

/// The Homebrew version whose on-disk state this engine emulates and is
/// differential-verified against. Bump ONLY after the differential oracle
/// (e2e) passes against the newer Homebrew.
pub const EMULATED_BREW_VERSION: &str = "6.0.17";

#[derive(Debug, Error)]
pub enum ReceiptError {
    #[error("cannot establish Homebrew receipt fact: {0}")]
    MissingFact(String),
    #[error("malformed Homebrew receipt at {path}: {source}")]
    Malformed {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("Homebrew receipt at {path} is missing required field {field}")]
    MissingField { path: PathBuf, field: String },
    #[error("failed to read Homebrew receipt at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct BuiltOn {
    pub os: String,
    pub os_version: String,
    pub cpu_family: String,
    pub xcode: Option<String>,
    pub clt: Option<String>,
    pub preferred_perl: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FormulaVersions {
    pub stable: Option<String>,
    pub head: Option<String>,
    pub version_scheme: u64,
    pub compatibility_version: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FormulaSource {
    pub spec: String,
    pub versions: FormulaVersions,
    pub path: Option<String>,
    pub tap_git_head: Option<String>,
    pub tap: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RuntimeDependency {
    pub full_name: String,
    pub version: String,
    pub revision: u64,
    pub bottle_rebuild: u64,
    pub pkg_version: String,
    pub declared_directly: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct FormulaReceipt {
    pub homebrew_version: String,
    pub used_options: Vec<String>,
    pub unused_options: Vec<String>,
    pub built_as_bottle: bool,
    pub poured_from_bottle: bool,
    pub loaded_from_api: bool,
    pub loaded_from_internal_api: bool,
    pub installed_on_request: bool,
    pub changed_files: Option<Vec<String>>,
    pub time: u64,
    pub source_modified_time: u64,
    pub compiler: String,
    pub aliases: Vec<String>,
    pub runtime_dependencies: Vec<RuntimeDependency>,
    pub source: FormulaSource,
    pub arch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub built_on: Option<BuiltOn>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CaskSource {
    pub tap: String,
    pub tap_git_head: Option<String>,
    pub version: String,
    pub path: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CaskReceipt {
    pub homebrew_version: String,
    pub loaded_from_api: bool,
    pub loaded_from_internal_api: bool,
    pub uninstall_flight_blocks: bool,
    pub installed_on_request: bool,
    pub time: u64,
    pub runtime_dependencies: Map<String, Value>,
    pub source: CaskSource,
    pub arch: String,
    pub uninstall_artifacts: Vec<Value>,
    pub built_on: BuiltOn,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct CaskConfig {
    pub default: Value,
    pub env: Value,
    pub explicit: Value,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn pretty_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    let mut bytes = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
    let mut serializer = serde_json::Serializer::with_formatter(&mut bytes, formatter);
    value.serialize(&mut serializer)?;
    Ok(bytes)
}

impl FormulaReceipt {
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        pretty_json_bytes(self)
    }
}

impl CaskReceipt {
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        pretty_json_bytes(self)
    }
}

impl CaskConfig {
    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

fn parse_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, ReceiptError> {
    let bytes = fs::read(path).map_err(|source| ReceiptError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| {
        if let Some(field) = source
            .to_string()
            .strip_prefix("missing field `")
            .and_then(|rest| rest.split('`').next())
        {
            ReceiptError::MissingField {
                path: path.to_path_buf(),
                field: field.to_string(),
            }
        } else {
            ReceiptError::Malformed {
                path: path.to_path_buf(),
                source,
            }
        }
    })
}

pub fn read_formula_receipt(keg: &Path) -> Result<FormulaReceipt, ReceiptError> {
    parse_file(&keg.join("INSTALL_RECEIPT.json"))
}

pub fn read_cask_receipt(caskroom_token_dir: &Path) -> Result<CaskReceipt, ReceiptError> {
    parse_file(&caskroom_token_dir.join(".metadata/INSTALL_RECEIPT.json"))
}

pub fn read_cask_config(caskroom_token_dir: &Path) -> Result<CaskConfig, ReceiptError> {
    parse_file(&caskroom_token_dir.join(".metadata/config.json"))
}

fn command_output(program: &str, args: &[&str]) -> Option<String> {
    let output = std::process::Command::new(program)
        .args(args)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(target_os = "macos")]
pub fn native_build_system_info() -> Result<BuiltOn, ReceiptError> {
    let product_version = command_output("/usr/bin/sw_vers", &["-productVersion"])
        .ok_or_else(|| ReceiptError::MissingFact("macOS product version".to_string()))?;
    let mut parts = product_version.split('.');
    let major = parts.next().unwrap_or_default();
    let minor = parts.next().unwrap_or_default();
    let os_version = if minor == "0" || minor.is_empty() {
        format!("macOS {major}")
    } else {
        format!("macOS {major}.{minor}")
    };
    let family = command_output("/usr/sbin/sysctl", &["-n", "hw.cpufamily"])
        .and_then(|raw| raw.parse::<i64>().ok())
        .map(|value| value as u32)
        .map(|value| match value {
            0x2c91a47e => "arm_typhoon",
            0x92fb37c8 => "arm_twister",
            0x67ceee93 => "arm_hurricane_zephyr",
            0xe81e7ef6 => "arm_monsoon_mistral",
            0x07d34b9f => "arm_vortex_tempest",
            0x462504d2 => "arm_lightning_thunder",
            0x573b5eec => "arm_firestorm_icestorm",
            0xda33d83d => "arm_blizzard_avalanche",
            0xfa33415e => "arm_ibiza",
            0x5f4dea93 => "arm_lobos",
            0x72015832 => "arm_palma",
            0x6f5129ac => "arm_donan",
            0x17d5b93a => "arm_brava",
            0x1d5a87e8 => "arm_hidra",
            0xf76c5b1a => "arm_sotra",
            _ => "dunno",
        })
        .unwrap_or("dunno")
        .to_string();
    let xcode = command_output("/usr/bin/xcodebuild", &["-version"]).and_then(|value| {
        value
            .lines()
            .next()?
            .strip_prefix("Xcode ")
            .map(str::to_string)
    });
    let clt = command_output(
        "/usr/sbin/pkgutil",
        &["--pkg-info=com.apple.pkg.CLTools_Executables"],
    )
    .and_then(|value| {
        value
            .lines()
            .find_map(|line| line.strip_prefix("version: ").map(str::to_string))
    });
    let preferred_perl = command_output("/usr/bin/perl", &["-e", "printf \"%vd\\n\", $^V"])
        .and_then(|value| {
            value
                .rsplit_once('.')
                .map(|(version, _)| version.to_string())
        })
        .ok_or_else(|| ReceiptError::MissingFact("preferred system Perl".to_string()))?;
    Ok(BuiltOn {
        os: "Macintosh".to_string(),
        os_version,
        cpu_family: family,
        xcode,
        clt,
        preferred_perl: Some(preferred_perl),
        extra: Map::new(),
    })
}

#[cfg(target_os = "linux")]
pub fn native_build_system_info() -> Result<BuiltOn, ReceiptError> {
    let os_version = command_output("uname", &["-r"])
        .ok_or_else(|| ReceiptError::MissingFact("Linux version".to_string()))?;
    Ok(BuiltOn {
        os: "Linux".to_string(),
        os_version,
        cpu_family: std::env::consts::ARCH.to_string(),
        xcode: None,
        clt: None,
        preferred_perl: None,
        extra: Map::new(),
    })
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn native_build_system_info() -> Result<BuiltOn, ReceiptError> {
    Err(ReceiptError::MissingFact(format!(
        "Homebrew build-system metadata is unsupported on {}",
        std::env::consts::OS
    )))
}

/// Finds the most recent metadata snapshot by Homebrew's sortable timestamp
/// directory name. Version directory names remain opaque.
pub fn newest_cask_metadata_dir(
    caskroom_token_dir: &Path,
    version: &str,
) -> Result<Option<PathBuf>, ReceiptError> {
    let root = caskroom_token_dir.join(".metadata").join(version);
    let entries = match fs::read_dir(&root) {
        Ok(entries) => entries,
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(ReceiptError::Io { path: root, source }),
    };
    let mut dirs = BTreeMap::new();
    for entry in entries {
        let entry = entry.map_err(|source| ReceiptError::Io {
            path: root.clone(),
            source,
        })?;
        if entry
            .file_type()
            .map_err(|source| ReceiptError::Io {
                path: entry.path(),
                source,
            })?
            .is_dir()
        {
            dirs.insert(entry.file_name(), entry.path());
        }
    }
    Ok(dirs.pop_last().map(|(_, path)| path))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORMULA: &[u8] = include_bytes!("testdata/ada-url-INSTALL_RECEIPT.json");
    const CASK: &[u8] = include_bytes!("testdata/codex-INSTALL_RECEIPT.json");
    const CONFIG: &[u8] = include_bytes!("testdata/codex-config.json");

    #[test]
    fn formula_fixture_round_trips_byte_stably() {
        let receipt: FormulaReceipt = serde_json::from_slice(FORMULA).unwrap();
        assert_eq!(receipt.to_json_bytes().unwrap(), FORMULA);
    }

    #[test]
    fn cask_fixture_round_trips_byte_stably() {
        let receipt: CaskReceipt = serde_json::from_slice(CASK).unwrap();
        assert_eq!(receipt.to_json_bytes().unwrap(), CASK);
        let config: CaskConfig = serde_json::from_slice(CONFIG).unwrap();
        assert_eq!(config.to_json_bytes().unwrap(), CONFIG);
    }

    #[test]
    fn missing_field_is_classified() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("INSTALL_RECEIPT.json"), b"{}").unwrap();
        assert!(matches!(
            read_formula_receipt(dir.path()),
            Err(ReceiptError::MissingField { .. })
        ));
    }

    #[test]
    fn malformed_json_is_classified() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".metadata")).unwrap();
        fs::write(dir.path().join(".metadata/INSTALL_RECEIPT.json"), b"{").unwrap();
        assert!(matches!(
            read_cask_receipt(dir.path()),
            Err(ReceiptError::Malformed { .. })
        ));
    }

    #[test]
    fn extra_keys_and_newer_version_are_preserved() {
        let mut value: Value = serde_json::from_slice(CASK).unwrap();
        value["homebrew_version"] = Value::String("7.1.2-99-gabcdef0".into());
        value["future_key"] = Value::String("future-value".into());
        let receipt: CaskReceipt = serde_json::from_value(value).unwrap();
        assert_eq!(receipt.homebrew_version, "7.1.2-99-gabcdef0");
        assert_eq!(receipt.extra["future_key"], "future-value");
    }

    #[test]
    fn built_on_absent_probe_values_are_preserved_as_null() {
        let value = serde_json::to_value(BuiltOn {
            os: "Linux".to_string(),
            os_version: "test".to_string(),
            cpu_family: "test".to_string(),
            xcode: None,
            clt: None,
            preferred_perl: None,
            extra: Map::new(),
        })
        .unwrap();

        assert!(value["xcode"].is_null());
        assert!(value["clt"].is_null());
        assert!(value["preferred_perl"].is_null());
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    #[test]
    fn unsupported_platform_fails_closed() {
        assert!(matches!(
            native_build_system_info(),
            Err(ReceiptError::MissingFact(message))
                if message.contains("unsupported")
        ));
    }

    #[test]
    fn metadata_timestamp_order_does_not_order_versions() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".metadata/nightly");
        fs::create_dir_all(root.join("20260807033635.774")).unwrap();
        fs::create_dir(root.join("20260808010000.001")).unwrap();
        assert_eq!(
            newest_cask_metadata_dir(dir.path(), "nightly").unwrap(),
            Some(root.join("20260808010000.001"))
        );
    }
}
