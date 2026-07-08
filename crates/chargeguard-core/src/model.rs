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
pub struct BatteryPower {
    pub microwatts: u64,
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
    pub power: Option<BatteryPower>,
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn battery_device_deserializes_when_power_field_is_missing() {
        let value = json!({
            "id": "BAT1",
            "nativePath": "/tmp/BAT1",
            "type": "Battery",
            "present": true,
            "status": "Charging",
            "capacity": 94,
            "manufacturer": "COSMX",
            "modelName": "L24X4PG7",
            "nativeMode": {
                "supported": true,
                "source": "charge_types",
                "current": "Long_Life",
                "currentLabel": "Long Life",
                "supportedModes": [
                    {"raw": "Long_Life", "label": "Long Life"}
                ],
                "writable": true,
                "message": null
            }
        });

        let device: BatteryDevice = serde_json::from_value(value).unwrap();

        assert_eq!(device.power, None);
    }

    #[test]
    fn battery_power_serializes_as_optional_microwatts() {
        let device = BatteryDevice {
            id: "BAT1".to_string(),
            native_path: PathBuf::from("/tmp/BAT1"),
            type_name: "Battery".to_string(),
            present: true,
            status: Some("Charging".to_string()),
            capacity: Some(94),
            manufacturer: Some("COSMX".to_string()),
            model_name: Some("L24X4PG7".to_string()),
            power: Some(BatteryPower {
                microwatts: 12_345_678,
            }),
            native_mode: NativeModeInfo {
                supported: true,
                source: Some("charge_types".to_string()),
                current: Some("Long_Life".to_string()),
                current_label: Some("Long Life".to_string()),
                supported_modes: vec![NativeMode {
                    raw: "Long_Life".to_string(),
                    label: "Long Life".to_string(),
                }],
                writable: true,
                message: None,
            },
        };

        let value = serde_json::to_value(device).unwrap();

        assert_eq!(value["power"]["microwatts"], 12_345_678);
    }
}
