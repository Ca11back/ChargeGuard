use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    device::PowerSupply,
    error::{ChargeGuardError, Result},
    model::{BatteryDevice, Diagnostics, NativeModeInfo, ObservedAttribute, RawPowerSupplyEntry},
    parser::{display_label, normalize_mode, parse_charge_types},
};

const DEFAULT_POWER_SUPPLY_ROOT: &str = "/sys/class/power_supply";
const NON_MVP_INFORMATIONAL_ATTRIBUTES: &[&str] = &[
    "charge_control_start_threshold",
    "charge_control_end_threshold",
    "conservation_mode",
    "charge_behaviour",
];
const RAW_ATTRIBUTES: &[&str] = &[
    "type",
    "present",
    "status",
    "capacity",
    "manufacturer",
    "model_name",
    "charge_type",
    "charge_types",
    "charge_control_start_threshold",
    "charge_control_end_threshold",
    "conservation_mode",
    "charge_behaviour",
];

#[derive(Debug, Clone)]
pub struct SysfsPowerSupply {
    root: PathBuf,
}

impl Default for SysfsPowerSupply {
    fn default() -> Self {
        Self::new(DEFAULT_POWER_SUPPLY_ROOT)
    }
}

impl SysfsPowerSupply {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn diagnostics(&self, version: impl Into<String>) -> Diagnostics {
        let devices = self.discover_devices().unwrap_or_default();
        let raw_entries = self.raw_entries().unwrap_or_default();
        let mut informational_attributes = Vec::new();
        for entry in &raw_entries {
            for name in NON_MVP_INFORMATIONAL_ATTRIBUTES {
                if let Some(Some(value)) = entry.attributes.get(*name) {
                    informational_attributes.push(ObservedAttribute {
                        device_id: entry.id.clone(),
                        name: (*name).to_string(),
                        value: Some(value.clone()),
                    });
                }
            }
        }

        Diagnostics {
            version: version.into(),
            kernel_version: kernel_version(),
            power_supply_root: self.root.clone(),
            devices,
            raw_entries,
            informational_attributes,
            service_status: None,
            polkit_available: None,
        }
    }

    pub fn raw_entries(&self) -> Result<Vec<RawPowerSupplyEntry>> {
        let entries =
            fs::read_dir(&self.root).map_err(|source| ChargeGuardError::SysfsReadFailed {
                path: self.root.clone(),
                source,
            })?;

        let mut raw_entries = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|source| ChargeGuardError::SysfsReadFailed {
                path: self.root.clone(),
                source,
            })?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let id = entry.file_name().to_string_lossy().into_owned();
            let mut attributes = BTreeMap::new();
            for attr in RAW_ATTRIBUTES {
                attributes.insert(
                    (*attr).to_string(),
                    read_optional_trimmed(&path.join(attr))?,
                );
            }

            let type_name = attributes.get("type").cloned().flatten();
            let present = attributes
                .get("present")
                .cloned()
                .flatten()
                .map(|value| value != "0");

            raw_entries.push(RawPowerSupplyEntry {
                id,
                path,
                type_name,
                present,
                attributes,
            });
        }

        raw_entries.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(raw_entries)
    }

    fn discover_one(&self, id: String, path: PathBuf) -> Result<Option<BatteryDevice>> {
        let type_path = path.join("type");
        let type_name = match read_optional_trimmed(&type_path)? {
            Some(value) => value,
            None => return Ok(None),
        };
        if type_name != "Battery" {
            return Ok(None);
        }

        let present = match read_optional_trimmed(&path.join("present"))? {
            Some(value) => value != "0",
            None => true,
        };
        if !present {
            return Ok(None);
        }

        let status = read_optional_trimmed(&path.join("status"))?;
        let capacity = read_optional_trimmed(&path.join("capacity"))?
            .and_then(|value| value.parse::<u8>().ok());
        let manufacturer = read_optional_trimmed(&path.join("manufacturer"))?;
        let model_name = read_optional_trimmed(&path.join("model_name"))?;
        let native_mode = self.native_mode_info(&id, &path)?;

        Ok(Some(BatteryDevice {
            id,
            native_path: path,
            type_name,
            present,
            status,
            capacity,
            manufacturer,
            model_name,
            native_mode,
        }))
    }

    fn native_mode_info(&self, id: &str, path: &Path) -> Result<NativeModeInfo> {
        let charge_types_path = path.join("charge_types");
        if let Some(raw_charge_types) = read_optional_trimmed(&charge_types_path)? {
            let parsed = parse_charge_types(&raw_charge_types).map_err(|message| {
                ChargeGuardError::ParseNativeModes {
                    path: charge_types_path.clone(),
                    message,
                }
            })?;

            let current_label = parsed.active_raw.as_deref().map(display_label);
            let writable = path.join("charge_type").exists() || path.join("charge_types").exists();

            return Ok(NativeModeInfo {
                supported: true,
                source: Some("charge_types".to_string()),
                current: parsed.active_raw,
                current_label,
                supported_modes: parsed.modes,
                writable,
                message: None,
            });
        }

        let charge_type_path = path.join("charge_type");
        if let Some(current) = read_optional_trimmed(&charge_type_path)? {
            return Ok(NativeModeInfo {
                supported: false,
                source: Some("charge_type".to_string()),
                current_label: Some(display_label(&current)),
                current: Some(current),
                supported_modes: Vec::new(),
                writable: false,
                message: Some(format!(
                    "Device '{id}' exposes current native mode but not enumerable writable modes"
                )),
            });
        }

        Ok(NativeModeInfo::unsupported(format!(
            "Device '{id}' does not expose charge_types or charge_type"
        )))
    }

    fn write_target(&self, path: &Path) -> Option<PathBuf> {
        let charge_type = path.join("charge_type");
        if charge_type.exists() {
            return Some(charge_type);
        }

        let charge_types = path.join("charge_types");
        if charge_types.exists() {
            return Some(charge_types);
        }

        None
    }
}

impl PowerSupply for SysfsPowerSupply {
    fn discover_devices(&self) -> Result<Vec<BatteryDevice>> {
        let entries =
            fs::read_dir(&self.root).map_err(|source| ChargeGuardError::SysfsReadFailed {
                path: self.root.clone(),
                source,
            })?;

        let mut devices = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|source| ChargeGuardError::SysfsReadFailed {
                path: self.root.clone(),
                source,
            })?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let id = entry.file_name().to_string_lossy().into_owned();
            if let Some(device) = self.discover_one(id, path)? {
                devices.push(device);
            }
        }

        devices.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(devices)
    }

    fn get_device(&self, id: &str) -> Result<BatteryDevice> {
        self.discover_devices()?
            .into_iter()
            .find(|device| device.id == id)
            .ok_or_else(|| ChargeGuardError::UnsupportedDevice(id.to_string()))
    }

    fn set_native_mode(&self, id: &str, mode: &str) -> Result<BatteryDevice> {
        let device = self.get_device(id)?;
        let device_path = device.native_path.clone();
        if !device.native_mode.supported {
            if device.native_mode.source.as_deref() == Some("charge_type") {
                return Err(ChargeGuardError::NativeModesNotEnumerable {
                    device_id: id.to_string(),
                });
            }
            return Err(ChargeGuardError::NoNativeModeInterface {
                device_id: id.to_string(),
            });
        }

        let requested = normalize_mode(mode);
        let selected = device
            .native_mode
            .supported_modes
            .iter()
            .find(|candidate| normalize_mode(&candidate.raw) == requested)
            .map(|candidate| candidate.raw.clone())
            .ok_or_else(|| ChargeGuardError::InvalidMode {
                device_id: id.to_string(),
                requested: mode.to_string(),
            })?;

        let target = self.write_target(&device_path).ok_or_else(|| {
            ChargeGuardError::NoNativeModeInterface {
                device_id: id.to_string(),
            }
        })?;

        fs::write(&target, selected.as_bytes()).map_err(|source| {
            ChargeGuardError::SysfsWriteFailed {
                path: target.clone(),
                source,
            }
        })?;

        let refreshed = self.get_device(id)?;
        let actual = refreshed.native_mode.current.clone();
        if actual.as_deref().map(normalize_mode).as_deref() != Some(requested.as_str()) {
            return Err(ChargeGuardError::ReadbackMismatch {
                device_id: id.to_string(),
                requested: selected,
                actual,
            });
        }

        Ok(refreshed)
    }
}

fn read_optional_trimmed(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(value) => Ok(Some(value.trim().to_string())),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(ChargeGuardError::SysfsReadFailed {
            path: path.to_path_buf(),
            source,
        }),
    }
}

fn kernel_version() -> Option<String> {
    fs::read_to_string("/proc/sys/kernel/osrelease")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use crate::NativeMode;
    #[cfg(unix)]
    use std::os::unix::fs as unix_fs;
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn discovers_present_battery_with_charge_types() {
        let fixture = Fixture::new();
        fixture.battery(
            "BAT1",
            &[
                ("type", "Battery"),
                ("present", "1"),
                ("status", "Charging"),
                ("capacity", "94"),
                ("manufacturer", "COSMX"),
                ("model_name", "L24X4PG7"),
                ("charge_types", "[Fast] Standard Long_Life"),
                ("charge_type", "Fast"),
            ],
        );
        fixture.entry("AC", &[("type", "Mains")]);

        let devices = fixture.sysfs().discover_devices().unwrap();

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, "BAT1");
        assert_eq!(devices[0].capacity, Some(94));
        assert_eq!(devices[0].native_mode.current.as_deref(), Some("Fast"));
        assert_eq!(
            raw_modes(&devices[0].native_mode.supported_modes),
            ["Fast", "Standard", "Long_Life"]
        );
    }

    #[test]
    fn treats_missing_present_as_present_and_present_zero_as_absent() {
        let fixture = Fixture::new();
        fixture.battery(
            "BAT0",
            &[("type", "Battery"), ("charge_types", "[Standard]")],
        );
        fixture.battery(
            "BAT1",
            &[
                ("type", "Battery"),
                ("present", "0"),
                ("charge_types", "[Fast] Standard"),
            ],
        );

        let devices = fixture.sysfs().discover_devices().unwrap();

        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, "BAT0");
    }

    #[test]
    fn reports_charge_type_without_charge_types_as_read_only() {
        let fixture = Fixture::new();
        fixture.battery("BAT1", &[("type", "Battery"), ("charge_type", "Long_Life")]);

        let device = fixture.sysfs().get_device("BAT1").unwrap();

        assert!(!device.native_mode.supported);
        assert!(!device.native_mode.writable);
        assert_eq!(device.native_mode.source.as_deref(), Some("charge_type"));
        assert_eq!(device.native_mode.current.as_deref(), Some("Long_Life"));
    }

    #[test]
    fn validates_mode_before_write_and_reads_back() {
        let fixture = Fixture::new();
        fixture.battery(
            "BAT1",
            &[
                ("type", "Battery"),
                ("charge_types", "[Fast] Standard Long_Life"),
                ("charge_type", "Fast"),
            ],
        );

        let result = fixture.sysfs().set_native_mode("BAT1", "long life");

        assert!(matches!(
            result.unwrap_err(),
            ChargeGuardError::ReadbackMismatch { .. }
        ));
        assert_eq!(
            fs::read_to_string(fixture.path("BAT1/charge_type")).unwrap(),
            "Long_Life"
        );
    }

    #[test]
    fn write_path_succeeds_when_readback_matches_requested_mode() {
        let fixture = Fixture::new();
        fixture.battery(
            "BAT1",
            &[
                ("type", "Battery"),
                ("charge_types", "[Fast] Standard Long_Life"),
                ("charge_type", "Fast"),
            ],
        );

        let device = fixture.sysfs().set_native_mode("BAT1", "fast").unwrap();

        assert_eq!(device.native_mode.current.as_deref(), Some("Fast"));
        assert_eq!(
            fs::read_to_string(fixture.path("BAT1/charge_type")).unwrap(),
            "Fast"
        );
    }

    #[cfg(unix)]
    #[test]
    fn write_path_accepts_symlinked_power_supply_entries() {
        let fixture = Fixture::new();
        fixture.symlinked_battery(
            "BAT1",
            &[
                ("type", "Battery"),
                ("charge_types", "[Fast] Standard Long_Life"),
                ("charge_type", "Fast"),
            ],
        );

        let device = fixture.sysfs().set_native_mode("BAT1", "fast").unwrap();

        assert_eq!(device.native_mode.current.as_deref(), Some("Fast"));
        assert_eq!(
            fs::read_to_string(fixture.path("real-devices/BAT1/charge_type")).unwrap(),
            "Fast"
        );
    }

    #[test]
    fn rejects_invalid_mode_before_write() {
        let fixture = Fixture::new();
        fixture.battery(
            "BAT1",
            &[
                ("type", "Battery"),
                ("charge_types", "[Fast] Standard"),
                ("charge_type", "Fast"),
            ],
        );

        let error = fixture
            .sysfs()
            .set_native_mode("BAT1", "Turbo")
            .unwrap_err();

        assert!(matches!(error, ChargeGuardError::InvalidMode { .. }));
        assert_eq!(
            fs::read_to_string(fixture.path("BAT1/charge_type")).unwrap(),
            "Fast"
        );
    }

    #[test]
    fn detects_informational_non_mvp_attributes() {
        let fixture = Fixture::new();
        fixture.battery(
            "BAT1",
            &[
                ("type", "Battery"),
                ("charge_types", "[Standard]"),
                ("charge_control_end_threshold", "80"),
            ],
        );

        let diagnostics = fixture.sysfs().diagnostics("0.1.0");

        assert_eq!(diagnostics.informational_attributes.len(), 1);
        assert_eq!(
            diagnostics.informational_attributes[0].name,
            "charge_control_end_threshold"
        );
    }

    fn raw_modes(modes: &[NativeMode]) -> Vec<&str> {
        modes.iter().map(|mode| mode.raw.as_str()).collect()
    }

    struct Fixture {
        temp: TempDir,
    }

    impl Fixture {
        fn new() -> Self {
            Self {
                temp: tempfile::tempdir().unwrap(),
            }
        }

        fn sysfs(&self) -> SysfsPowerSupply {
            SysfsPowerSupply::new(self.temp.path())
        }

        fn path(&self, relative: &str) -> PathBuf {
            self.temp.path().join(relative)
        }

        fn battery(&self, id: &str, attributes: &[(&str, &str)]) {
            self.entry(id, attributes);
        }

        #[cfg(unix)]
        fn symlinked_battery(&self, id: &str, attributes: &[(&str, &str)]) {
            let real_path = self.temp.path().join("real-devices").join(id);
            fs::create_dir_all(&real_path).unwrap();
            for (name, value) in attributes {
                write_attr(&real_path, name, value);
            }

            unix_fs::symlink(real_path, self.temp.path().join(id)).unwrap();
        }

        fn entry(&self, id: &str, attributes: &[(&str, &str)]) {
            let path = self.temp.path().join(id);
            fs::create_dir_all(&path).unwrap();
            for (name, value) in attributes {
                write_attr(&path, name, value);
            }
        }
    }

    fn write_attr(path: &Path, name: &str, value: &str) {
        fs::write(path.join(name), value).unwrap();
    }
}
