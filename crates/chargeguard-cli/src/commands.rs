use std::collections::HashMap;

use chargeguard_core::{
    BatteryDevice, ChargeGuardError, Diagnostics, INTERFACE_NAME, OBJECT_PATH, SERVICE_NAME,
};
use clap::{Parser, Subcommand};
use serde::de::DeserializeOwned;
use thiserror::Error;
use zbus::zvariant::OwnedValue;

pub type Result<T> = std::result::Result<T, CliError>;
type PropertyMap = HashMap<String, OwnedValue>;

const ERROR_PREFIX: &str = "io.github.ChargeGuard1.Error.";

#[derive(Debug, Parser)]
#[command(name = "chargeguard")]
#[command(about = "Manage native Linux battery charging modes")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    List,
    Current {
        device: Option<String>,
    },
    Modes {
        device: Option<String>,
    },
    Set {
        mode: String,
        #[arg(long)]
        device: Option<String>,
    },
    Doctor {
        #[arg(long)]
        json: bool,
    },
    Raw {
        device: Option<String>,
    },
}

pub struct ChargeGuardClient {
    connection: zbus::Connection,
}

impl ChargeGuardClient {
    pub async fn connect() -> Result<Self> {
        let connection = zbus::Connection::system().await?;

        Ok(Self { connection })
    }

    pub async fn list_devices(&self) -> Result<Vec<BatteryDevice>> {
        let reply = self
            .connection
            .call_method(
                Some(SERVICE_NAME),
                OBJECT_PATH,
                Some(INTERFACE_NAME),
                "ListDevices",
                &(),
            )
            .await?;
        let maps = reply.body().deserialize::<Vec<PropertyMap>>()?;
        maps.iter().map(from_json_field).collect()
    }

    pub async fn get_device(&self, id: &str) -> Result<BatteryDevice> {
        let reply = self
            .connection
            .call_method(
                Some(SERVICE_NAME),
                OBJECT_PATH,
                Some(INTERFACE_NAME),
                "GetDevice",
                &(id),
            )
            .await?;
        let map = reply.body().deserialize::<PropertyMap>()?;
        from_json_field(&map)
    }

    pub async fn set_native_mode(&self, id: &str, mode: &str) -> Result<BatteryDevice> {
        let reply = self
            .connection
            .call_method(
                Some(SERVICE_NAME),
                OBJECT_PATH,
                Some(INTERFACE_NAME),
                "SetNativeMode",
                &(id, mode),
            )
            .await?;
        let map = reply.body().deserialize::<PropertyMap>()?;
        from_json_field(&map)
    }

    pub async fn diagnostics(&self) -> Result<Diagnostics> {
        let reply = self
            .connection
            .call_method(
                Some(SERVICE_NAME),
                OBJECT_PATH,
                Some(INTERFACE_NAME),
                "GetDiagnostics",
                &(),
            )
            .await?;
        let map = reply.body().deserialize::<PropertyMap>()?;
        from_json_field(&map)
    }

    pub async fn resolve_device(&self, device: Option<String>) -> Result<BatteryDevice> {
        let id = self.resolve_device_id(device).await?;
        self.get_device(&id).await
    }

    pub async fn resolve_device_id(&self, device: Option<String>) -> Result<String> {
        if let Some(device) = device {
            return Ok(device);
        }

        let devices = self.list_devices().await?;
        match devices.as_slice() {
            [] => Err(ChargeGuardError::UnsupportedDevice(
                "no present batteries were discovered".to_string(),
            )
            .into()),
            [device] => Ok(device.id.clone()),
            _ => Err(CliError::AmbiguousDevice),
        }
    }
}

fn from_json_field<T: DeserializeOwned>(map: &PropertyMap) -> Result<T> {
    let value = map
        .get("json")
        .ok_or_else(|| CliError::Protocol("missing D-Bus json field".to_string()))?;
    let json = String::try_from(value.clone())
        .map_err(|error| CliError::Protocol(format!("invalid D-Bus json field: {error}")))?;
    Ok(serde_json::from_str(&json)?)
}

#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    Core(#[from] ChargeGuardError),

    #[error("D-Bus error: {0}")]
    Zbus(#[from] zbus::Error),

    #[error("D-Bus decode error: {0}")]
    Zvariant(#[from] zbus::zvariant::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("multiple batteries were discovered; pass --device or DEVICE")]
    AmbiguousDevice,

    #[error("{0}")]
    Protocol(String),
}

impl CliError {
    pub fn code(&self) -> &'static str {
        match self {
            CliError::Core(error) => error.code(),
            CliError::Zbus(error) => code_from_zbus_error(error),
            CliError::Protocol(_) | CliError::Zvariant(_) | CliError::Json(_) => "internal_error",
            CliError::AmbiguousDevice => "ambiguous_device",
        }
    }
}

fn code_from_zbus_error(error: &zbus::Error) -> &'static str {
    match error {
        zbus::Error::MethodError(name, _, _) => code_from_remote_name(name.as_str()),
        _ => "internal_error",
    }
}

fn code_from_remote_name(name: &str) -> &'static str {
    match name {
        "org.freedesktop.DBus.Error.ServiceUnknown"
        | "org.freedesktop.DBus.Error.NameHasNoOwner" => "service_unavailable",
        "io.github.ChargeGuard1.Error.UnsupportedDevice" => "unsupported_device",
        "io.github.ChargeGuard1.Error.NoNativeModeInterface" => "no_native_mode_interface",
        "io.github.ChargeGuard1.Error.NativeModesNotEnumerable" => "native_modes_not_enumerable",
        "io.github.ChargeGuard1.Error.InvalidMode" => "invalid_mode",
        "io.github.ChargeGuard1.Error.PermissionDenied" => "permission_denied",
        "io.github.ChargeGuard1.Error.SysfsReadFailed" => "sysfs_read_failed",
        "io.github.ChargeGuard1.Error.SysfsWriteFailed" => "sysfs_write_failed",
        "io.github.ChargeGuard1.Error.ReadbackMismatch" => "readback_mismatch",
        "io.github.ChargeGuard1.Error.ServiceUnavailable" => "service_unavailable",
        "io.github.ChargeGuard1.Error.InternalError" => "internal_error",
        _ if name.starts_with(ERROR_PREFIX) => "internal_error",
        _ => "internal_error",
    }
}

pub fn format_error(error: &CliError) -> String {
    format!("{}: {error}", error.code())
}

#[cfg(test)]
mod tests {
    use std::{io, path::PathBuf};

    use super::*;
    use zbus::zvariant::Value;

    fn string_value(value: &str) -> OwnedValue {
        OwnedValue::try_from(Value::from(value)).expect("test string is a valid D-Bus value")
    }

    #[test]
    fn decodes_exact_remote_error_names() {
        let cases = [
            (
                "io.github.ChargeGuard1.Error.UnsupportedDevice",
                "unsupported_device",
            ),
            (
                "io.github.ChargeGuard1.Error.NoNativeModeInterface",
                "no_native_mode_interface",
            ),
            (
                "io.github.ChargeGuard1.Error.NativeModesNotEnumerable",
                "native_modes_not_enumerable",
            ),
            ("io.github.ChargeGuard1.Error.InvalidMode", "invalid_mode"),
            (
                "io.github.ChargeGuard1.Error.PermissionDenied",
                "permission_denied",
            ),
            (
                "io.github.ChargeGuard1.Error.SysfsReadFailed",
                "sysfs_read_failed",
            ),
            (
                "io.github.ChargeGuard1.Error.SysfsWriteFailed",
                "sysfs_write_failed",
            ),
            (
                "io.github.ChargeGuard1.Error.ReadbackMismatch",
                "readback_mismatch",
            ),
            (
                "io.github.ChargeGuard1.Error.ServiceUnavailable",
                "service_unavailable",
            ),
            (
                "io.github.ChargeGuard1.Error.InternalError",
                "internal_error",
            ),
            (
                "org.freedesktop.DBus.Error.ServiceUnknown",
                "service_unavailable",
            ),
            (
                "org.freedesktop.DBus.Error.NameHasNoOwner",
                "service_unavailable",
            ),
            ("io.github.ChargeGuard1.Error.FutureError", "internal_error"),
            ("org.freedesktop.DBus.Error.Failed", "internal_error"),
        ];

        for (name, expected) in cases {
            assert_eq!(code_from_remote_name(name), expected, "{name}");
        }
    }

    #[test]
    fn protocol_and_local_errors_have_stable_prefixes() {
        let cases = [
            (
                CliError::Protocol("missing D-Bus json field".to_string()),
                "internal_error: missing D-Bus json field",
            ),
            (
                CliError::AmbiguousDevice,
                "ambiguous_device: multiple batteries were discovered; pass --device or DEVICE",
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(format_error(&error), expected);
        }
    }

    #[test]
    fn every_core_error_renders_its_exact_stderr_prefix() {
        let path = PathBuf::from("/sys/class/power_supply/BAT0/charge_type");
        let cases = [
            (
                ChargeGuardError::UnsupportedDevice("BAT0".to_string()),
                "unsupported_device: ",
            ),
            (
                ChargeGuardError::NoNativeModeInterface {
                    device_id: "BAT0".to_string(),
                },
                "no_native_mode_interface: ",
            ),
            (
                ChargeGuardError::NativeModesNotEnumerable {
                    device_id: "BAT0".to_string(),
                },
                "native_modes_not_enumerable: ",
            ),
            (
                ChargeGuardError::InvalidMode {
                    device_id: "BAT0".to_string(),
                    requested: "Turbo".to_string(),
                },
                "invalid_mode: ",
            ),
            (
                ChargeGuardError::PermissionDenied("denied".to_string()),
                "permission_denied: ",
            ),
            (
                ChargeGuardError::SysfsReadFailed {
                    path: path.clone(),
                    source: io::Error::other("read failed"),
                },
                "sysfs_read_failed: ",
            ),
            (
                ChargeGuardError::SysfsWriteFailed {
                    path: path.clone(),
                    source: io::Error::other("write failed"),
                },
                "sysfs_write_failed: ",
            ),
            (
                ChargeGuardError::ReadbackMismatch {
                    device_id: "BAT0".to_string(),
                    requested: "Fast".to_string(),
                    actual: Some("Standard".to_string()),
                },
                "readback_mismatch: ",
            ),
            (
                ChargeGuardError::ServiceUnavailable("missing".to_string()),
                "service_unavailable: ",
            ),
            (
                ChargeGuardError::ParseNativeModes {
                    path: path.clone(),
                    message: "bad modes".to_string(),
                },
                "internal_error: ",
            ),
            (
                ChargeGuardError::Internal("unexpected".to_string()),
                "internal_error: ",
            ),
        ];

        for (error, expected_prefix) in cases {
            let rendered = format_error(&CliError::Core(error));
            assert!(
                rendered.starts_with(expected_prefix),
                "expected {expected_prefix:?}, got {rendered:?}"
            );
        }
    }

    #[test]
    fn malformed_payloads_are_internal_errors() {
        let missing = PropertyMap::new();
        let error = from_json_field::<serde_json::Value>(&missing).unwrap_err();
        assert_eq!(error.code(), "internal_error");

        let mut wrong_type = PropertyMap::new();
        wrong_type.insert("json".to_string(), OwnedValue::from(42u32));
        let error = from_json_field::<serde_json::Value>(&wrong_type).unwrap_err();
        assert_eq!(error.code(), "internal_error");

        let mut invalid_json = PropertyMap::new();
        invalid_json.insert("json".to_string(), string_value("{"));
        let error = from_json_field::<serde_json::Value>(&invalid_json).unwrap_err();
        assert_eq!(error.code(), "internal_error");
    }

    #[test]
    fn non_method_dbus_failures_are_internal_errors() {
        let error = CliError::Zbus(zbus::Error::Failure("transport failed".to_string()));
        assert_eq!(error.code(), "internal_error");
        assert_eq!(
            format_error(&error),
            "internal_error: D-Bus error: transport failed"
        );
    }
}
