use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};

pub const SERVICE_NAME: &str = "io.github.ChargeGuard1";
pub const OBJECT_PATH: &str = "/io/github/ChargeGuard1";
pub const INTERFACE_NAME: &str = "io.github.ChargeGuard1.Manager";
pub const POLKIT_ACTION_SET_NATIVE_MODE: &str = "io.github.ChargeGuard1.set-native-mode";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeMode {
    pub raw: String,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeModeInfo {
    pub supported: bool,
    pub source: Option<String>,
    pub current: Option<String>,
    pub current_label: Option<String>,
    pub supported_modes: Vec<NativeMode>,
    pub writable: bool,
    pub message: Option<String>,
}

impl NativeModeInfo {
    pub fn unsupported(message: impl Into<String>) -> Self {
        Self {
            supported: false,
            source: None,
            current: None,
            current_label: None,
            supported_modes: Vec::new(),
            writable: false,
            message: Some(message.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryDevice {
    pub id: String,
    pub native_path: PathBuf,
    #[serde(rename = "type")]
    pub type_name: String,
    pub present: bool,
    pub status: Option<String>,
    pub capacity: Option<u8>,
    pub manufacturer: Option<String>,
    pub model_name: Option<String>,
    pub native_mode: NativeModeInfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawPowerSupplyEntry {
    pub id: String,
    pub path: PathBuf,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub present: Option<bool>,
    pub attributes: BTreeMap<String, Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservedAttribute {
    pub device_id: String,
    pub name: String,
    pub value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    pub version: String,
    pub kernel_version: Option<String>,
    pub power_supply_root: PathBuf,
    pub devices: Vec<BatteryDevice>,
    pub raw_entries: Vec<RawPowerSupplyEntry>,
    pub informational_attributes: Vec<ObservedAttribute>,
    pub service_status: Option<String>,
    pub polkit_available: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedChargeTypes {
    pub modes: Vec<NativeMode>,
    pub active_raw: Option<String>,
}
